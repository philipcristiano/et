use sqlx::postgres::PgPool;
#[derive(sqlx::FromRow)]
pub struct SFAccountTXQueryResultRow {
    posted: chrono::NaiveDateTime,
    transacted_at: Option<chrono::NaiveDateTime>,
    description: String,
    amount: sqlx::postgres::types::PgMoney,
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
        SELECT sat.posted, sat.transacted_at, sat.amount, sat.description
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
        SELECT sat.posted, sat.transacted_at, sat.amount, sat.description
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
               tr{
                 @if let Some(transacted_at) = tx.transacted_at {
                     td { (transacted_at) }
                 } @else {
                     td {(tx.posted)}
                 }
                 td { (tx.description)}
                 td { (tx.amount.to_decimal(2))}
               }
               }
               }

           }
        }
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
