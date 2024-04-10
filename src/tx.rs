use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Form,
};
use futures::try_join;
use maud::Render;
use sqlx::postgres::PgPool;

pub type TransactionID = String;
#[derive(sqlx::FromRow)]
pub struct SFAccountTXQueryResultRow {
    id: String,
    posted: chrono::NaiveDateTime,
    transacted_at: Option<chrono::NaiveDateTime>,
    description: String,
    amount: sqlx::postgres::types::PgMoney,
    connection_id: crate::SFConnectionID,
    account_id: crate::accounts::AccountID,
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
                        //hx-from={ "find #search-input-" (self.id)}
                        //hx-from={ "(find #search-input-" (self.id) ")"}
                        hx-from="input"
                        hx-get="/f/labels/search"
                        hx-target={"#search-results-" (self.id)}
                        hx-trigger="input changed delay:100ms from:input"
                   {
                    input #{ "search-input-" (self.id)}
                        hx-get="/f/labels/search"
                        name="search"
                        placeholder="Begin typing to search labels"
                    {}
                    input type="hidden" name="connection_id" value={(self.connection_id)} {}
                    input type="hidden" name="account_id" value={(self.account_id)} {}
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
            hx-get={"/f/transactions/" (self.connection_id) "/" (self.account_id) "/" (self.id) "/edit" }
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
        SELECT sat.posted, sat.transacted_at, sat.amount, sat.description, sat.connection_id, sat.id, sat.account_id
        FROM simplefin_accounts sa
            JOIN simplefin_account_transactions sat
            ON sa.id = sat.account_id
            AND sa.connection_id = sat.connection_id
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
        SELECT sat.posted, sat.transacted_at, sat.amount, sat.description, sat.connection_id, sat.id, sat.account_id
        FROM simplefin_account_transactions sat
        WHERE
            connection_id = $1
        AND account_id = $2
        AND id = $3
            "#,
            params.connection_id,
            params.account_id,
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
        if let Some(account_id) = params.account_id {
            Self::by_account_id(account_id, pool).await
        } else {
            Self::all(pool).await
        }
    }

    #[tracing::instrument]
    pub async fn by_account_id(
        account_id: crate::accounts::AccountID,
        pool: &PgPool,
    ) -> anyhow::Result<Self> {
        let res = sqlx::query_as!(
            SFAccountTXQueryResultRow,
            r#"
        SELECT sat.posted, sat.transacted_at, sat.amount, sat.description, sat.connection_id, sat.account_id, sat.id
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

#[derive(Clone, Debug)]
pub struct AccountTransactionLabel {
    connection_id: crate::SFConnectionID,
    account_id: crate::accounts::AccountID,
    transaction_id: TransactionID,
    label_id: crate::labels::LabelID,
}

impl From<TXAddLabelPost> for AccountTransactionLabel {
    fn from(item: TXAddLabelPost) -> Self {
        AccountTransactionLabel {
            connection_id: item.full_transaction_id.connection_id,
            account_id: item.full_transaction_id.account_id,
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
    INSERT INTO transaction_labels ( connection_id, account_id, transaction_id, label_id )
    VALUES ( $1, $2, $3, $4 )
    ON CONFLICT (account_id, connection_id, transaction_id, label_id) DO NOTHING
            "#,
            self.connection_id,
            self.account_id,
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
    account_id: String,
    connection_id: uuid::Uuid,
    id: String,
    posted: chrono::DateTime<chrono::Utc>,
    transacted_at: Option<chrono::DateTime<chrono::Utc>>,
    amount: rust_decimal::Decimal,
    pending: Option<bool>,
    description: String,
}

impl SFAccountTransaction {
    #[tracing::instrument]
    pub async fn ensure_in_db(self, pool: &PgPool) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
    INSERT INTO simplefin_account_transactions ( connection_id, account_id, id, posted, amount, transacted_at, pending, description )
    VALUES ( $1, $2, $3, $4, $5, $6, $7, $8 )
    ON CONFLICT (account_id, connection_id, id) DO NOTHING
            "#,
            self.connection_id,
            self.account_id,
            self.id,
            self.posted.naive_utc(),
            sqlx::postgres::types::PgMoney::from_decimal(self.amount, 2),
            self.transacted_at.map(|ta| ta.naive_utc()),
            self.pending,
            self.description
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub fn from_transaction(
        act: &crate::accounts::SFAccount,
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
    pub connection_id: crate::SFConnectionID,
    pub account_id: crate::accounts::AccountID,
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
