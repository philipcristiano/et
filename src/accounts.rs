use sqlx::postgres::PgPool;

pub type AccountID = uuid::Uuid;
#[derive(Debug)]
pub struct SFAccount {
    pub simplefin_id: String,
    pub connection_id: crate::ConnectionID,
    pub currency: String,
    pub name: String,
}
impl SFAccount {
    #[tracing::instrument]
    pub async fn ensure_in_db(self, pool: &PgPool) -> anyhow::Result<Account> {
        let res = sqlx::query_as!(
            Account,
            r#"
    INSERT INTO simplefin_accounts ( connection_id, simplefin_id, name, currency )
    VALUES ( $1, $2, $3, $4 )
    ON CONFLICT (connection_id, simplefin_id) DO UPDATE set name = EXCLUDED.name
    RETURNING id, connection_id, currency, name
            "#,
            self.connection_id,
            self.simplefin_id,
            self.name,
            self.currency,
        )
        .fetch_one(pool)
        .await?;

        Ok(res)
    }
}

pub struct Account {
    pub id: AccountID,
    pub connection_id: crate::ConnectionID,
    pub currency: String,
    pub name: String,
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

#[derive(Debug)]
pub struct SFAccountBalance {
    pub account_id: AccountID,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub balance: rust_decimal::Decimal,
}
impl SFAccountBalance {
    #[tracing::instrument]
    pub async fn ensure_in_db(&self, pool: &PgPool) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
    INSERT INTO simplefin_account_balances ( account_id, ts, balance )
    VALUES ( $1, $2, $3 )
    ON CONFLICT (account_id, ts) DO UPDATE set balance = EXCLUDED.balance
            "#,
            self.account_id,
            self.timestamp,
            sqlx::postgres::types::PgMoney::from_decimal(self.balance, 2),
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
