use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Form,
};
use futures::try_join;
use maud::Render;
use sqlx::postgres::types::{PgLQuery, PgLQueryLevel};
use sqlx::postgres::PgPool;
use std::str::FromStr;

pub type TransactionID = uuid::Uuid;
#[derive(sqlx::FromRow)]
pub struct SFAccountTXQueryResultRow {
    id: String,
    posted: chrono::DateTime<chrono::Utc>,
    transacted_at: Option<chrono::DateTime<chrono::Utc>>,
    description: String,
    amount: sqlx::postgres::types::PgMoney,
    account_id: crate::accounts::AccountID,
}

#[derive(sqlx::FromRow)]
pub struct SFAccountTXAmountQueryResultRow {
    amount: Option<sqlx::postgres::types::PgMoney>,
}

impl maud::Render for SFAccountTXAmountQueryResultRow {
    fn render(&self) -> maud::Markup {
        maud::html! {
              @if let Some(amount) = self.amount {
                  span {"Amount: " (amount.to_decimal(2))}
              } @else {
                  span {"No records found"}
              }
        }
    }
}
impl SFAccountTXQueryResultRow {
    fn render_edit(&self, labels_markup: maud::Markup) -> maud::Markup {
        maud::html! {
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
                  form
                        hx-get="/f/labels/search"
                        hx-target={"#search-results-" (self.id)}
                        hx-trigger="input changed delay:100ms from:input"
                   {
                    input #{ "search-input-" (self.id)}
                        hx-get="/f/labels/search"
                        name="search"
                        placeholder="Begin typing to search labels"
                    {}
                    input type="hidden" name="transaction_id" value={(self.id)} {}

                    ul #{"search-results-" (self.id)} {}
                  }

             }
            div {
                span {"Current labels:"}
                (labels_markup)}
        }}
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
                  td { (transacted_at) }
              } @else {
                  td {(self.posted)}
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
    pub async fn all(pool: &PgPool) -> anyhow::Result<Self> {
        let res = sqlx::query_as!(
            SFAccountTXQueryResultRow,
            r#"
        SELECT sat.posted, sat.transacted_at, sat.amount, sat.description, sat.id, sat.account_id
        FROM simplefin_accounts sa
            JOIN simplefin_account_transactions sat
            ON sa.id = sat.account_id
        ORDER BY
            sat.posted DESC
            "#,
        )
        .fetch_all(pool)
        .await?;
        Ok(res.into())
    }
    pub async fn one(
        params: &FullTransactionID,
        pool: &PgPool,
    ) -> anyhow::Result<SFAccountTXQueryResultRow> {
        let res = sqlx::query_as!(
            SFAccountTXQueryResultRow,
            r#"
        SELECT sat.posted, sat.transacted_at, sat.amount, sat.description, sat.id, sat.account_id
        FROM simplefin_account_transactions sat
        WHERE id = $1
            "#,
            params.transaction_id
        )
        .fetch_one(pool)
        .await?;
        Ok(res)
    }

    #[tracing::instrument]
    pub async fn from_options(
        params: crate::TransactionsFilterOptions,
        pool: &PgPool,
    ) -> anyhow::Result<Self> {
        if let Some(label) = params.labeled {
            return Self::with_label(label, pool).await;
        } else if let Some(label) = params.not_labeled {
            return Self::without_label(label, pool).await;
        } else if let Some(account_id) = params.account_id {
            return Self::by_account_id(account_id, pool).await;
        } else {
            return Self::all(pool).await;
        }
        return Err(anyhow::anyhow!("Not implemented"));
    }

    #[tracing::instrument]
    pub async fn amount_from_options(
        params: crate::TransactionsFilterOptions,
        pool: &PgPool,
    ) -> anyhow::Result<SFAccountTXAmountQueryResultRow> {
        if let Some(label) = params.labeled {
            return Self::amount_with_label(label, params.start_datetime, pool).await;
        }
        return Err(anyhow::anyhow!("Not implemented"));
    }

    #[tracing::instrument]
    pub async fn by_account_id(
        account_id: crate::accounts::AccountID,
        pool: &PgPool,
    ) -> anyhow::Result<Self> {
        let res = sqlx::query_as!(
            SFAccountTXQueryResultRow,
            r#"
        SELECT sat.posted, sat.transacted_at, sat.amount, sat.description, sat.account_id, sat.id
        FROM simplefin_account_transactions sat
        WHERE sat.account_id = $1
        ORDER BY
            sat.posted DESC
            "#,
            account_id
        )
        .fetch_all(pool)
        .await?;

        Ok(res.into())
    }

    #[tracing::instrument]
    pub async fn amount_with_label(
        label: String,
        start_datetime: chrono::DateTime<chrono::Utc>,
        pool: &PgPool,
    ) -> anyhow::Result<SFAccountTXAmountQueryResultRow> {
        let query_levels = string_label_to_plquerylevels(label)?;
        let query = PgLQuery::from(query_levels);
        Self::tx_label_amount_query(query, start_datetime, pool).await
    }

    #[tracing::instrument]
    async fn tx_label_amount_query(
        q: sqlx::postgres::types::PgLQuery,
        start_datetime: chrono::DateTime<chrono::Utc>,
        pool: &PgPool,
    ) -> anyhow::Result<SFAccountTXAmountQueryResultRow> {
        let res = sqlx::query_as!(
            SFAccountTXAmountQueryResultRow,
            r#"
        SELECT sum(sat.amount) as amount
        FROM simplefin_account_transactions sat
        JOIN transaction_labels tl
            ON sat.id = tl.transaction_id
        JOIN labels l
            ON tl.label_id = l.id
        WHERE l.label ~ $1
        AND sat.posted >= $2
            "#,
            q,
            start_datetime
        )
        .fetch_one(pool)
        .await?;

        Ok(res)
    }

    #[tracing::instrument]
    pub async fn with_label(label: String, pool: &PgPool) -> anyhow::Result<Self> {
        let query_levels = string_label_to_plquerylevels(label)?;
        let query = PgLQuery::from(query_levels);
        Self::tx_label_query(query, pool).await
    }

    #[tracing::instrument]
    pub async fn without_label(label: String, pool: &PgPool) -> anyhow::Result<Self> {
        let query_levels = string_label_to_plquerylevels(label)?;
        let query = PgLQuery::from(query_levels);
        Self::tx_not_label_query(query, pool).await
    }

    #[tracing::instrument]
    async fn tx_label_query(
        q: sqlx::postgres::types::PgLQuery,
        pool: &PgPool,
    ) -> anyhow::Result<Self> {
        let res = sqlx::query_as!(
            SFAccountTXQueryResultRow,
            r#"
        SELECT sat.posted, sat.transacted_at, sat.amount, sat.description, sat.account_id, sat.id
        FROM simplefin_account_transactions sat
        JOIN transaction_labels tl
            ON sat.id = tl.transaction_id
        JOIN labels l
            ON tl.label_id = l.id
        WHERE l.label ~ $1
        ORDER BY
            sat.posted DESC
            "#,
            q
        )
        .fetch_all(pool)
        .await?;

        Ok(res.into())
    }
    #[tracing::instrument]
    async fn tx_not_label_query(
        q: sqlx::postgres::types::PgLQuery,
        pool: &PgPool,
    ) -> anyhow::Result<Self> {
        let res = sqlx::query_as!(
            SFAccountTXQueryResultRow,
            r#"
        SELECT sat.posted, sat.transacted_at, sat.amount, sat.description, sat.account_id, sat.id
        FROM simplefin_account_transactions sat
        LEFT OUTER JOIN (
            SELECT transaction_id, label_id
            FROM transaction_labels stl
            JOIN labels sl
              ON stl.label_id = sl.id
            WHERE sl.label ~ $1
        ) AS tl
        ON sat.id = tl.transaction_id
        WHERE tl.transaction_id IS NULL
        ORDER BY
            sat.posted DESC
            "#,
            q
        )
        .fetch_all(pool)
        .await?;

        Ok(res.into())
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

fn string_label_to_plquerylevels(label: String) -> anyhow::Result<Vec<PgLQueryLevel>> {
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

impl From<TXAddLabelPost> for AccountTransactionLabel {
    fn from(item: TXAddLabelPost) -> Self {
        AccountTransactionLabel {
            transaction_id: item.full_transaction_id.transaction_id,
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
    connection_id: uuid::Uuid,
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
    ON CONFLICT (account_id, simplefin_id) DO UPDATE set pending = EXCLUDED.pending
    RETURNING id, account_id, posted, amount, transacted_at, pending, description
            "#,
            self.account_id,
            self.id,
            self.posted,
            sqlx::postgres::types::PgMoney::from_decimal(self.amount, 2),
            self.transacted_at,
            self.pending,
            self.description
        )
        .fetch_one(pool)
        .await?;

        Ok(())
    }

    pub fn from_transaction(
        act: &crate::accounts::Account,
        tx: &crate::simplefin_api::Transaction,
    ) -> Self {
        SFAccountTransaction {
            account_id: act.id.clone(),
            connection_id: act.connection_id,
            id: tx.id.clone(),
            posted: tx.posted,
            transacted_at: tx.transacted_at,
            amount: tx.amount,
            pending: tx.pending,
            description: tx.description.clone(),
        }
    }
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct FullTransactionID {
    pub transaction_id: TransactionID,
}
pub async fn handle_tx_edit_get(
    State(app_state): State<crate::AppState>,
    _user: service_conventions::oidc::OIDCUser,
    Path(params): Path<FullTransactionID>,
) -> Result<Response, crate::AppError> {
    let full_tx_id: FullTransactionID = params.into();

    let row_f = SFAccountTXQuery::one(&full_tx_id, &app_state.db);
    let labels_f = crate::labels::LabelsQuery::for_tx(&full_tx_id, &app_state.db);

    let (row, labels) = try_join!(row_f, labels_f)?;

    let r = row.render_edit(labels.render_as_table_for_tx(full_tx_id));
    Ok(r.into_response())
}
pub async fn handle_tx_edit_post(
    State(app_state): State<crate::AppState>,
    _user: service_conventions::oidc::OIDCUser,
    Path(params): Path<FullTransactionID>,
) -> Result<Response, crate::AppError> {
    let full_tx_id: FullTransactionID = params.into();
    let row = SFAccountTXQuery::one(&full_tx_id, &app_state.db).await?;

    Ok(row.render().into_response())
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct TXAddLabelPost {
    label_id: crate::labels::LabelID,
    #[serde(flatten)]
    full_transaction_id: crate::tx::FullTransactionID,
}

pub async fn handle_tx_add_label(
    State(app_state): State<crate::AppState>,
    _user: service_conventions::oidc::OIDCUser,

    Form(form): Form<TXAddLabelPost>,
) -> Result<Response, crate::AppError> {
    let ftxid = form.full_transaction_id.clone();

    let tx_label: AccountTransactionLabel = form.into();
    tx_label.ensure_in_db(&app_state.db).await?;

    let labels = crate::labels::LabelsQuery::for_tx(&ftxid, &app_state.db).await?;

    Ok(labels.render_as_table_for_tx(ftxid).into_response())
}

pub async fn handle_tx_delete_label(
    State(app_state): State<crate::AppState>,
    _user: service_conventions::oidc::OIDCUser,

    Form(form): Form<TXAddLabelPost>,
) -> Result<Response, crate::AppError> {
    let ftxid = form.full_transaction_id.clone();

    let tx_label: AccountTransactionLabel = form.into();
    tx_label.delete_in_db(&app_state.db).await?;

    let labels = crate::labels::LabelsQuery::for_tx(&ftxid, &app_state.db).await?;

    Ok(labels.render_as_table_for_tx(ftxid).into_response())
}
