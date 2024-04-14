use crate::{AppError, AppState};
use sqlx::postgres::PgPool;

use axum::{
    extract::{FromRef, Path, State},
    response::{IntoResponse, Response},
    Form,
};

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
    RETURNING id, connection_id, currency, name, active
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

#[derive(Clone, Debug, serde::Deserialize)]
pub struct Account {
    pub id: AccountID,
    pub connection_id: crate::ConnectionID,
    pub currency: String,
    pub name: String,
    pub active: bool,
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

impl Account {
    #[tracing::instrument]
    pub async fn get_all(pool: &PgPool) -> anyhow::Result<Vec<Self>> {
        let res = sqlx::query_as!(
            Self,
            r#"
        SELECT id, connection_id, currency, name, active
        FROM simplefin_accounts
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(res)
    }
    pub async fn get(account_id: AccountID, pool: &PgPool) -> anyhow::Result<Option<Self>> {
        let res = sqlx::query_as!(
            Self,
            r#"
        SELECT id, connection_id, currency, name, active
        FROM simplefin_accounts
        WHERE id = $1
            "#,
            account_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(res)
    }

    pub async fn enable_sync(account_id: AccountID, pool: &PgPool) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
        UPDATE simplefin_accounts
        SET active = true
        WHERE id = $1
            "#,
            account_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn disable_sync(account_id: AccountID, pool: &PgPool) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
        UPDATE simplefin_accounts
        SET active = false
        WHERE id = $1
            "#,
            account_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub fn render_edit(&self) -> maud::Markup {
        maud::html!( {
            div #{"account-" (self.id)}{
            (self.id)
                {
                @if self.active {
                    div
                        hx-delete={"/f/accounts/" (self.id) "/active"}
                        hx-target={"#account-" (self.id)}
                        hx-trigger="click"
                        { "Sync enabled. [CLICK HERE] to disable"}
                } @else {
                    div
                        hx-post={"/f/accounts/" (self.id) "/active"}
                        hx-target={"#account-" (self.id)}
                        hx-trigger="click"
                        { "Sync disabled. [CLICK HERE] to enable"}
                }
                div
                    hx-post={"/f/accounts/" (self.id) "/delete-transactions"}
                    hx-target={"#account-" (self.id)}
                    hx-confirm="Delete all accounts?"
                    hx-trigger="click"
                    { "DELETE ALL TRANSACTIONS (requires confirmation)"}

            }}
        })
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

#[derive(serde::Deserialize, Clone, Debug)]
pub struct AccountIDFilter {
    pub account_id: AccountID,
}

pub async fn get_accounts_f(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
) -> Result<Response, AppError> {
    let accounts = Account::get_all(&app_state.db).await?;
    let resp = maud::html! {
        @for account in accounts {
            (account.render_edit())
        }

    };
    Ok(resp.into_response())
}

fn render_maybe_edit(maybe_account: Option<Account>) -> Result<Response, AppError> {
    if let Some(account) = maybe_account {
        let resp = maud::html! {
            (account.render_edit())
        }
        .into_response();
        Ok(resp)
    } else {
        Ok((http::StatusCode::NOT_FOUND, format!("Not Found")).into_response())
    }
}

pub async fn get_account_f(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
    Path(params): Path<AccountIDFilter>,
) -> Result<Response, AppError> {
    let account_id = params.account_id;
    let maybe_account = Account::get(account_id, &app_state.db).await?;
    render_maybe_edit(maybe_account)
}
pub async fn handle_active_post(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
    Path(params): Path<AccountIDFilter>,
) -> Result<Response, AppError> {
    let account_id = params.account_id;
    Account::enable_sync(account_id, &app_state.db).await?;
    let maybe_account = Account::get(account_id, &app_state.db).await?;
    render_maybe_edit(maybe_account)
}

pub async fn handle_active_delete(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
    Path(params): Path<AccountIDFilter>,
) -> Result<Response, AppError> {
    let account_id = params.account_id;
    Account::disable_sync(account_id, &app_state.db).await?;
    let maybe_account = Account::get(account_id, &app_state.db).await?;
    render_maybe_edit(maybe_account)
}

pub async fn handle_delete_transactions(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
    Path(params): Path<AccountIDFilter>,
) -> Result<Response, AppError> {
    let account_id = params.account_id;
    crate::tx::AccountTransaction::delete_for_account(account_id, &app_state.db).await?;
    let maybe_account = Account::get(account_id, &app_state.db).await?;
    render_maybe_edit(maybe_account)
}

pub async fn get_account(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
    Path(params): Path<AccountIDFilter>,
) -> Result<Response, AppError> {
    let account_id = params.account_id;
    let maybe_account_f = Account::get(account_id, &app_state.db);

    let user_connections_f = crate::Connection::connections(&app_state.db);
    let balances_f = SFAccountBalanceQueryResult::get_balances(&app_state.db);
    let (user_connections, balances, maybe_account) =
        futures::try_join!(user_connections_f, balances_f, maybe_account_f)?;
    if let Some(account) = maybe_account {
        Ok(crate::html::maud_page(maud::html! {
              div class="flex flex-col lg:flex-row"{
              (crate::html::sidebar(user_connections, balances))
              div #main class="main" {
                    (account.render_edit())
              }}

        })
        .into_response())
    } else {
        Ok((http::StatusCode::NOT_FOUND, format!("Not Found")).into_response())
    }
}
