use crate::{TransactionFilter, TransactionFilterComponent, TransactionsFilterOptions};
use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
    Form,
};
use futures::try_join;
use maud::Render;
use sqlx::postgres::types::{PgLQuery, PgLQueryLevel};
use sqlx::postgres::PgPool;
use std::str::FromStr;

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

pub type TransactionID = uuid::Uuid;
#[derive(sqlx::FromRow)]
pub struct SFAccountTXQueryResultRow {
    id: TransactionID,
    posted: chrono::DateTime<chrono::Utc>,
    transacted_at: Option<chrono::DateTime<chrono::Utc>>,
    description: String,
    amount: sqlx::postgres::types::PgMoney,
    account_id: crate::accounts::AccountID,
}
#[derive(sqlx::FromRow, Clone, Debug)]
pub struct SFAccountTXGroupedQueryResultRow {
    pub interval: Option<chrono::DateTime<chrono::Utc>>,
    pub amount: Option<sqlx::postgres::types::PgMoney>,
}

impl SFAccountTXGroupedQueryResultRow {
    pub fn amount(&self) -> Option<Decimal> {
        Some(self.amount?.to_decimal(2))
    }
    pub fn amount_f32(&self) -> Option<f32> {
        self.amount?.to_decimal(2).to_f32()
    }
    pub fn name(&self) -> Option<String> {
        Some(self.interval?.format("%m/%d/%Y").to_string())
    }
}

#[derive(sqlx::FromRow)]
pub struct SFAccountTXAmountQueryResultRow {
    amount: Option<sqlx::postgres::types::PgMoney>,
}

impl maud::Render for SFAccountTXAmountQueryResultRow {
    fn render(&self) -> maud::Markup {
        maud::html! {
              @if let Some(amount) = self.amount {
                  span {(amount.to_decimal(2))}
              } @else {
                  span {"-"}
              }
        }
    }
}

pub fn label_search_box(
    id: &String,
    txf: crate::TransactionFilter,
) -> anyhow::Result<maud::Markup> {
    let _qs = txf.clone();
    //TODO: use txf to construct input fields
    Ok(maud::html! {
    form
          hx-get={"/f/labels/search" }
          hx-target={"#search-results-" (id)}
          hx-trigger={"input changed delay:100ms throttle:50ms from:input#search-input-" (id)}
     {
      input #{ "search-input-" (id)}
          name="search"
          placeholder="Begin typing to search labels"
      {}
      (txf.render_to_hidden_input_fields())

      ul #{"search-results-" (id)} {}
    }})
}
impl SFAccountTXQueryResultRow {
    fn render_edit(
        &self,
        labels_markup: maud::Markup,
        account: crate::accounts::Account,
    ) -> anyhow::Result<maud::Markup> {
        let f = crate::TransactionFilter::default().with_transaction_id(self.id)?;
        Ok(maud::html! {
         tr hx-target="this"
            hx-swap="outerHTML"
            {
              @if let Some(transacted_at) = self.transacted_at {
                  td { (transacted_at) }
              } @else {
                  td {(self.posted)}
              }
              td { (self.description)}
              td { (self.amount.to_decimal(2))}
        }
        td {
              div {
                (label_search_box(&self.id.to_string(), f)?)
             }
            div {
                span {"Current labels:"}
                (labels_markup)}
        }
        td {
            div {
                (account.preffered_name())
             }
        }
        })
    }
}

impl maud::Render for SFAccountTXQueryResultRow {
    fn render(&self) -> maud::Markup {
        maud::html! {
         tr
            hx-target="this"
            hx-swap="outerHTML"
            hx-trigger="click"
            hx-get={"/f/transactions/"  (self.id) "/edit" }
            {
              @if let Some(transacted_at) = self.transacted_at {
                  td { (transacted_at.date_naive()) }
              } @else {
                  td {(self.posted.date_naive())}
              }
              td { (self.description)}
              td { (self.amount.to_decimal(2))}
        }}
    }
}

pub struct SFAccountTXQuery {
    item: Vec<SFAccountTXQueryResultRow>,
}

impl From<Vec<SFAccountTXQueryResultRow>> for SFAccountTXQuery {
    fn from(item: Vec<SFAccountTXQueryResultRow>) -> Self {
        SFAccountTXQuery { item }
    }
}

impl SFAccountTXQuery {
    #[tracing::instrument]
    pub async fn all(
        start_datetime: Option<chrono::DateTime<chrono::Utc>>,
        end_datetime: Option<chrono::DateTime<chrono::Utc>>,
        pool: &PgPool,
    ) -> anyhow::Result<Self> {
        let res = sqlx::query_as!(
            SFAccountTXQueryResultRow,
            r#"
        SELECT sat.posted, sat.transacted_at, sat.amount, sat.description, sat.id, sat.account_id
        FROM simplefin_accounts sa
            JOIN simplefin_account_transactions sat
            ON sa.id = sat.account_id
        WHERE
        ($1::timestamptz IS NULL OR sat.transacted_at >= $1)
        AND ($2::timestamptz IS NULL OR sat.transacted_at < $2)
        ORDER BY
            sat.transacted_at DESC
            "#,
            start_datetime,
            end_datetime,
        )
        .fetch_all(pool)
        .await?;
        Ok(res.into())
    }
    #[tracing::instrument]
    pub async fn one(
        txid: &TransactionID,
        pool: &PgPool,
    ) -> anyhow::Result<SFAccountTXQueryResultRow> {
        let res = sqlx::query_as!(
            SFAccountTXQueryResultRow,
            r#"
        SELECT sat.posted, sat.transacted_at, sat.amount, sat.description, sat.id, sat.account_id
        FROM simplefin_account_transactions sat
        WHERE id = $1
            "#,
            txid
        )
        .fetch_one(pool)
        .await?;
        Ok(res)
    }

    #[tracing::instrument]
    pub async fn from_filter_options(
        tfo: &TransactionsFilterOptions,
        pool: &PgPool,
    ) -> anyhow::Result<Self> {
        let q = if let Some(label) = tfo.labeled.clone() {
            let query_levels = string_label_to_plquerylevels(label)?;
            Some(PgLQuery::from(query_levels))
        } else {
            None
        };
        let not_label_q: Option<PgLQuery> = if let Some(label) = tfo.not_labeled.clone() {
            let query_levels = string_label_to_plquerylevels(label)?;
            Some(PgLQuery::from(query_levels))
            //return Err(anyhow::anyhow!("Not-labeled not supported"))
        } else {
            None
        };

        let description_q = if let Some(df) = tfo.description_contains.clone() {
            Some(format!("%{df}%"))
        } else {
            None
        };

        //if let Some(_) = tfo.not_labeled.clone() {
        //    return Err(anyhow::anyhow!("not-labeled is not yet supported"))
        //}
        tracing::info!(lquery=?q, not_lquery=?not_label_q, description_q=?description_q, "Query Filter Options");

        let res = sqlx::query_as!(
            SFAccountTXQueryResultRow,
            r#"
        SELECT sat.posted, sat.transacted_at, sat.amount, sat.description, sat.account_id, sat.id
        FROM simplefin_account_transactions sat

        WHERE ($1::lquery IS NULL OR sat.id IN (
            SELECT tl_inner.transaction_id FROM transaction_labels tl_inner
            JOIN labels l_inner ON tl_inner.label_id = l_inner.id
            WHERE tl_inner.transaction_id = sat.id AND l_inner.label ~ $1
        ))
        AND ($2::timestamptz IS NULL OR sat.transacted_at >= $2)
        AND ($3::timestamptz IS NULL OR sat.transacted_at < $3)
        AND ($4::uuid IS NULL OR sat.account_id = $4)
        AND ($5::uuid IS NULL OR sat.id = $5)
        AND ($6::text IS NULL or sat.description LIKE $6)
        AND NOT EXISTS (
            SELECT 1
            FROM transaction_labels tl2
            JOIN labels l2 ON tl2.label_id = l2.id
            WHERE tl2.transaction_id = sat.id
            AND ($7::lquery IS NOT NULL AND l2.label ~ $7)
        )
        ORDER BY
            sat.transacted_at DESC
            "#,
            q,
            tfo.start_datetime,
            tfo.end_datetime,
            tfo.account_id,
            tfo.transaction_id,
            description_q,
            not_label_q,
        )
        .fetch_all(pool)
        .await?;

        Ok(res.into())
    }

    #[tracing::instrument]
    pub async fn from_options_group_by(
        tfo: &TransactionsFilterOptions,
        pool: &PgPool,
    ) -> anyhow::Result<Vec<SFAccountTXGroupedQueryResultRow>> {
        if let Some(aid) = tfo.account_id {
            return crate::accounts::SFAccountBalance::by_date(aid, tfo, pool).await;
        }
        let q = if let Some(label) = tfo.labeled.clone() {
            let query_levels = string_label_to_plquerylevels(label)?;
            Some(PgLQuery::from(query_levels))
        } else {
            None
        };
        let not_label_q: Option<PgLQuery> = if let Some(label) = tfo.not_labeled.clone() {
            let query_levels = string_label_to_plquerylevels(label)?;
            Some(PgLQuery::from(query_levels))
            //return Err(anyhow::anyhow!("Not-labeled not supported"))
        } else {
            None
        };

        let description_q = if let Some(df) = tfo.description_contains.clone() {
            Some(format!("%{df}%"))
        } else {
            None
        };

        //if let Some(_) = tfo.not_labeled.clone() {
        //    return Err(anyhow::anyhow!("not-labeled is not yet supported"))
        //}
        tracing::info!(lquery=?q, not_lquery=?not_label_q, description_q=?description_q, "Query Filter Options");

        let res = sqlx::query_as!(
            SFAccountTXGroupedQueryResultRow,
            r#"

        WITH daily_totals AS (
            SELECT
                DATE_TRUNC('day', sat.transacted_at) as interval,
                SUM(sat.amount) as daily_sum
            FROM simplefin_account_transactions sat

            WHERE ($1::lquery IS NULL OR sat.id IN (
                SELECT tl_inner.transaction_id FROM transaction_labels tl_inner
                JOIN labels l_inner ON tl_inner.label_id = l_inner.id
                WHERE tl_inner.transaction_id = sat.id AND l_inner.label ~ $1
            ))
            AND ($2::timestamptz IS NULL OR sat.transacted_at >= $2)
            AND ($3::timestamptz IS NULL OR sat.transacted_at < $3)
            AND ($4::uuid IS NULL OR sat.account_id = $4)
            AND ($5::uuid IS NULL OR sat.id = $5)
            AND ($6::text IS NULL or sat.description LIKE $6)
            AND NOT EXISTS (
                SELECT 1
                FROM transaction_labels tl2
                JOIN labels l2 ON tl2.label_id = l2.id
                WHERE tl2.transaction_id = sat.id
                AND ($7::lquery IS NOT NULL AND l2.label ~ $7)
            )

            GROUP BY DATE_TRUNC('day', sat.transacted_at)

        )
        SELECT
            interval,
            SUM(daily_sum) OVER (ORDER BY interval) AS amount
        FROM daily_totals
        ORDER BY interval ASC;
            "#,
            q,
            tfo.start_datetime,
            tfo.end_datetime,
            tfo.account_id,
            tfo.transaction_id,
            description_q,
            not_label_q,
        )
        .fetch_all(pool)
        .await?;

        Ok(res.into())
    }

    #[tracing::instrument]
    pub async fn amount_from_filter_options(
        tfo: &TransactionsFilterOptions,
        pool: &PgPool,
    ) -> anyhow::Result<SFAccountTXAmountQueryResultRow> {
        let q = if let Some(label) = tfo.labeled.clone() {
            let query_levels = string_label_to_plquerylevels(label)?;
            Some(PgLQuery::from(query_levels))
        } else {
            None
        };
        let not_label_q: Option<PgLQuery> = if let Some(label) = tfo.not_labeled.clone() {
            let query_levels = string_label_to_plquerylevels(label)?;
            Some(PgLQuery::from(query_levels))
            //return Err(anyhow::anyhow!("Not-labeled not supported"))
        } else {
            None
        };

        let description_q = if let Some(df) = tfo.description_contains.clone() {
            Some(format!("%{df}%"))
        } else {
            None
        };

        tracing::info!(lquery=?q, not_lquery=?not_label_q, description_q=?description_q, "Query Filter Options");

        let res = sqlx::query_as!(
            SFAccountTXAmountQueryResultRow,
            r#"
        SELECT sum(sat.amount) as amount
        FROM simplefin_account_transactions sat

        WHERE ($1::lquery IS NULL OR sat.id IN (
            SELECT tl_inner.transaction_id FROM transaction_labels tl_inner
            JOIN labels l_inner ON tl_inner.label_id = l_inner.id
            WHERE tl_inner.transaction_id = sat.id AND l_inner.label ~ $1
        ))
        AND ($2::timestamptz IS NULL OR sat.transacted_at >= $2)
        AND ($3::timestamptz IS NULL OR sat.transacted_at < $3)
        AND ($4::uuid IS NULL OR sat.account_id = $4)
        AND ($5::uuid IS NULL OR sat.id = $5)
        AND ($6::text IS NULL or sat.description LIKE $6)
        AND NOT EXISTS (
            SELECT 1
            FROM transaction_labels tl2
            JOIN labels l2 ON tl2.label_id = l2.id
            WHERE tl2.transaction_id = sat.id
            AND ($7::lquery IS NOT NULL AND l2.label ~ $7)
        )
            "#,
            q,
            tfo.start_datetime,
            tfo.end_datetime,
            tfo.account_id,
            tfo.transaction_id,
            description_q,
            not_label_q,
        )
        .fetch_one(pool)
        .await?;
        Ok(res)
    }
}

impl maud::Render for SFAccountTXQuery {
    fn render(&self) -> maud::Markup {
        maud::html! {
           h2 { "Transactions:" }
           table #transaction-table class="table-auto"{
               thead {
                 tr {
                     th { "Date"}
                     th { "Description"}
                     th { "Amount"}
                 }
               }
               tbody {
               @for tx in &self.item {
                  (tx)
               }
               }

           }
        }
    }
}

fn string_label_to_plquerylevels(label: crate::Label) -> anyhow::Result<Vec<PgLQueryLevel>> {
    let label_string_parts = label.split(".");
    let mut query_parts: Vec<PgLQueryLevel> = Vec::new();
    for string_part in label_string_parts {
        let ql = PgLQueryLevel::from_str(&string_part)?;
        query_parts.push(ql)
    }
    query_parts.push(PgLQueryLevel::from_str("*")?);
    Ok(query_parts)
}

#[derive(Clone, Debug)]
pub struct AccountTransactionLabel {
    transaction_id: TransactionID,
    label_id: crate::labels::LabelID,
}

impl From<TXAddLabelPostByTXID> for AccountTransactionLabel {
    fn from(item: TXAddLabelPostByTXID) -> Self {
        AccountTransactionLabel {
            transaction_id: item.transaction_id,
            label_id: item.label_id,
        }
    }
}

impl AccountTransactionLabel {
    #[tracing::instrument]
    pub async fn ensure_in_db(self, pool: &PgPool) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
    INSERT INTO transaction_labels ( transaction_id, label_id )
    VALUES ( $1, $2 )
    ON CONFLICT (transaction_id, label_id) DO NOTHING
            "#,
            self.transaction_id,
            self.label_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
    pub async fn delete_in_db(self, pool: &PgPool) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
    DELETE FROM transaction_labels
    WHERE transaction_id = $1
    AND label_id = $2
            "#,
            self.transaction_id,
            self.label_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
#[derive(Clone, Debug)]
pub struct SFAccountTransaction {
    account_id: crate::accounts::AccountID,
    connection_id: crate::ConnectionID,
    id: String,
    posted: chrono::DateTime<chrono::Utc>,
    transacted_at: Option<chrono::DateTime<chrono::Utc>>,
    amount: rust_decimal::Decimal,
    pending: Option<bool>,
    description: String,
}
pub type AccountTransactionID = uuid::Uuid;
#[derive(Clone, Debug)]
pub struct AccountTransaction {
    account_id: crate::accounts::AccountID,
    id: AccountTransactionID,
    posted: chrono::DateTime<chrono::Utc>,
    transacted_at: Option<chrono::DateTime<chrono::Utc>>,
    amount: sqlx::postgres::types::PgMoney,
    pending: Option<bool>,
    description: String,
}

impl AccountTransaction {
    pub async fn delete_for_account(
        account_id: crate::accounts::AccountID,
        pool: &PgPool,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
        DELETE
        FROM simplefin_account_transactions
        WHERE account_id = $1
            "#,
            account_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

impl SFAccountTransaction {
    #[tracing::instrument]
    pub async fn ensure_in_db(self, pool: &PgPool) -> anyhow::Result<()> {
        sqlx::query_as!(
            AccountTransaction,
            r#"
    INSERT INTO simplefin_account_transactions ( account_id, simplefin_id, posted, amount, transacted_at, pending, description )
    VALUES ( $1, $2, $3, $4, $5, $6, $7 )
    ON CONFLICT (account_id, simplefin_id)
        DO UPDATE
            SET
                posted = EXCLUDED.posted,
                amount = EXCLUDED.amount,
                transacted_at = EXCLUDED.transacted_at,
                pending = EXCLUDED.pending,
                description = EXCLUDED.description
            "#,
            self.account_id,
            self.id,
            self.posted,
            sqlx::postgres::types::PgMoney::from_decimal(self.amount, 2),
            self.transacted_at,
            self.pending,
            self.description
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub fn from_transaction(
        act: &crate::accounts::Account,
        tx: &crate::simplefin_api::Transaction,
    ) -> Self {
        SFAccountTransaction {
            account_id: act.id.clone(),
            connection_id: act.connection_id.clone(),
            id: tx.id.clone(),
            posted: tx.posted,
            transacted_at: tx.transacted_at,
            amount: tx.amount,
            pending: tx.pending,
            description: tx.description.clone(),
        }
    }
}

//TODO: This will be replace with TransactionFilter for handle_tx_edit_post to eventually support
//tagging multiple transactions in a query
#[derive(serde::Deserialize, Clone, Debug)]
pub struct FullTransactionID {
    pub transaction_id: TransactionID,
}
pub async fn handle_tx_edit_get(
    State(app_state): State<crate::AppState>,
    _user: service_conventions::oidc::OIDCUser,
    Path(params): Path<FullTransactionID>,
) -> Result<Response, crate::AppError> {
    let tx_id = params.transaction_id;

    let row_f = SFAccountTXQuery::one(&tx_id, &app_state.db);
    let labels_f = crate::labels::LabelsQuery::for_tx(&tx_id, &app_state.db);
    let account_f = crate::accounts::Account::get_for_tx_id(&tx_id, &app_state.db);

    let (row, labels, account_option) = try_join!(row_f, labels_f, account_f)?;

    if let Some(account) = account_option {
        let r = row.render_edit(labels.render_as_table_for_tx(tx_id), account)?;
        return Ok(r.into_response());
    } else {
        return Err(crate::AppError(anyhow::anyhow!("Account not found")));
    }
}
pub async fn handle_tx_edit_post(
    State(app_state): State<crate::AppState>,
    _user: service_conventions::oidc::OIDCUser,
    Path(params): Path<FullTransactionID>,
) -> Result<Response, crate::AppError> {
    let tx_id = params.transaction_id;
    let row = SFAccountTXQuery::one(&tx_id, &app_state.db).await?;

    Ok(row.render().into_response())
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct TXAddLabelPost {
    label_id: crate::labels::LabelID,
    #[serde(flatten)]
    transaction_filter: crate::TransactionsFilterOptions,
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct TXAddLabelPostByTXID {
    label_id: crate::labels::LabelID,
    transaction_id: crate::tx::TransactionID,
}

pub async fn handle_tx_add_label(
    State(app_state): State<crate::AppState>,
    _user: service_conventions::oidc::OIDCUser,

    Form(form): Form<TXAddLabelPost>,
) -> Result<Response, crate::AppError> {
    let filters: crate::TransactionFilter = form.transaction_filter.into();
    //let transactions = SFAccountTXQuery::from_options(filters, &app_state.db).await?;
    let transactions =
        SFAccountTXQuery::from_filter_options(&filters.into(), &app_state.db).await?;
    let mut last_tx = None;
    for t in transactions.item {
        let atl = AccountTransactionLabel {
            transaction_id: t.id,
            label_id: form.label_id,
        };
        atl.ensure_in_db(&app_state.db).await?;
        last_tx = Some(t)
    }

    if let Some(tx) = last_tx {
        let labels = crate::labels::LabelsQuery::for_tx(&tx.id, &app_state.db).await?;

        Ok(labels.render_as_table_for_tx(tx.id).into_response())
    } else {
        Ok((http::StatusCode::NOT_FOUND, format!("Not Found")).into_response())
    }
}

pub async fn handle_tx_delete_label(
    State(app_state): State<crate::AppState>,
    _user: service_conventions::oidc::OIDCUser,

    Query(inputs): Query<TXAddLabelPostByTXID>,
) -> Result<Response, crate::AppError> {
    let ftxid = inputs.transaction_id;

    let tx_label: AccountTransactionLabel = inputs.into();
    tx_label.delete_in_db(&app_state.db).await?;

    let labels = crate::labels::LabelsQuery::for_tx(&ftxid, &app_state.db).await?;

    Ok(labels.render_as_table_for_tx(ftxid).into_response())
}
