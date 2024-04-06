use sqlx::postgres::PgPool;

#[derive(Debug)]
pub struct SFAccount {
    pub id: String,
    pub connection_id: uuid::Uuid,
    pub currency: String,
    pub name: String,
}
impl SFAccount {
    #[tracing::instrument]
    pub async fn ensure_in_db(&self, pool: &PgPool) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
    INSERT INTO simplefin_accounts ( connection_id, id, name, currency )
    VALUES ( $1, $2, $3, $4 )
    ON CONFLICT (id, connection_id) DO NOTHING
            "#,
            self.connection_id,
            self.id,
            self.name,
            self.currency,
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

pub struct SFAccountBalanceQueryResult {
    pub name: String,
    pub account_id: AccountID,
    pub balance: sqlx::postgres::types::PgMoney,
}
impl SFAccountBalanceQueryResult {
    #[tracing::instrument]
    pub async fn get_balances(pool: &PgPool) -> anyhow::Result<Vec<SFAccountBalanceQueryResult>> {
        let res = sqlx::query_as!(
            SFAccountBalanceQueryResult,
            r#"
        SELECT name, sab.account_id, sab.balance
        FROM simplefin_accounts sa
        JOIN (
                SELECT account_id, max(ts) as ts
                FROM simplefin_account_balances
                GROUP BY (account_id)
            ) as last_ts
            ON sa.id = last_ts.account_id
        LEFT JOIN simplefin_account_balances sab
            ON last_ts.account_id = sab.account_id
            AND last_ts.ts = sab.ts;
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(res)
    }
}

pub type AccountID = String;
#[derive(Debug)]
pub struct SFAccountBalance {
    pub account_id: AccountID,
    pub connection_id: uuid::Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub balance: rust_decimal::Decimal,
}
impl SFAccountBalance {
    #[tracing::instrument]
    pub async fn ensure_in_db(&self, pool: &PgPool) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
    INSERT INTO simplefin_account_balances ( connection_id, account_id, ts, balance )
    VALUES ( $1, $2, $3, $4 )
    ON CONFLICT (account_id, connection_id, ts) DO NOTHING
            "#,
            self.connection_id,
            self.account_id,
            self.timestamp.naive_utc(),
            sqlx::postgres::types::PgMoney::from_decimal(self.balance, 2),
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
