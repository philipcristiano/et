use crate::{AppError, AppState};
use sqlx::postgres::PgPool;

use axum::{
    extract::{Path, State},
    response::{IntoResponse, Response},
    Form,
};
use axum_extra::extract::Query;

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
    RETURNING id, connection_id, currency, name, active, custom_name
            "#,
            self.connection_id.to_uuid(),
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
    pub custom_name: Option<String>,
    pub active: bool,
}

pub struct SFAccountBalanceQueryResult {
    pub name: String,
    pub custom_name: Option<String>,
    pub account_id: AccountID,
    pub balance: sqlx::postgres::types::PgMoney,
}
impl SFAccountBalanceQueryResult {
    #[tracing::instrument]
    pub async fn get_balances(pool: &PgPool) -> anyhow::Result<Vec<SFAccountBalanceQueryResult>> {
        let res = sqlx::query_as!(
            SFAccountBalanceQueryResult,
            r#"
        SELECT name, custom_name, sab.account_id, sab.balance
        FROM simplefin_accounts sa
        JOIN (
                SELECT account_id, max(ts) as ts
                FROM simplefin_account_balances
                GROUP BY (account_id)
            ) as last_ts
            ON sa.id = last_ts.account_id
        LEFT JOIN simplefin_account_balances sab
            ON last_ts.account_id = sab.account_id
            AND last_ts.ts = sab.ts
        ORDER BY
            COALESCE(custom_name, name),
            balance
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(res)
    }
    pub async fn get_active_balances(
        pool: &PgPool,
    ) -> anyhow::Result<Vec<SFAccountBalanceQueryResult>> {
        SFAccountBalanceQueryResult::get_balances_activity_state(pool, true).await
    }
    pub async fn get_inactive_balances(
        pool: &PgPool,
    ) -> anyhow::Result<Vec<SFAccountBalanceQueryResult>> {
        SFAccountBalanceQueryResult::get_balances_activity_state(pool, false).await
    }

    async fn get_balances_activity_state(
        pool: &PgPool,
        active: bool,
    ) -> anyhow::Result<Vec<SFAccountBalanceQueryResult>> {
        let res = sqlx::query_as!(
            SFAccountBalanceQueryResult,
            r#"
        SELECT name, custom_name, sab.account_id, sab.balance
        FROM simplefin_accounts sa
        JOIN (
                SELECT account_id, max(ts) as ts
                FROM simplefin_account_balances
                GROUP BY (account_id)
            ) as last_ts
            ON sa.id = last_ts.account_id
        LEFT JOIN simplefin_account_balances sab
            ON last_ts.account_id = sab.account_id
            AND last_ts.ts = sab.ts
        WHERE sa.active = $1
        ORDER BY
            COALESCE(custom_name, name),
            balance
            "#,
            active
        )
        .fetch_all(pool)
        .await?;

        Ok(res)
    }
}

impl Account {
    pub fn preffered_name(&self) -> String {
        if let Some(n) = self.custom_name.clone() {
            n
        } else {
            self.name.clone()
        }
    }
    #[tracing::instrument]
    pub async fn get_all(pool: &PgPool) -> anyhow::Result<Vec<Self>> {
        let res = sqlx::query_as!(
            Self,
            r#"
        SELECT id, connection_id, currency, name, active, custom_name
        FROM simplefin_accounts
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(res)
    }
    #[tracing::instrument]
    pub async fn get(account_id: AccountID, pool: &PgPool) -> anyhow::Result<Option<Self>> {
        let res = sqlx::query_as!(
            Self,
            r#"
        SELECT id, connection_id, currency, name, active, custom_name
        FROM simplefin_accounts
        WHERE id = $1
        ORDER BY
            COALESCE(custom_name, name)
            "#,
            account_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(res)
    }

    #[tracing::instrument]
    pub async fn get_for_tx_id(
        tx_id: &crate::tx::AccountTransactionID,
        pool: &PgPool,
    ) -> anyhow::Result<Option<Self>> {
        let res = sqlx::query_as!(
            Self,
            r#"
        SELECT sa.id, sa.connection_id, sa.currency, sa.name, sa.active, sa.custom_name
        FROM simplefin_accounts AS sa
        JOIN simplefin_account_transactions AS sat
        ON sat.account_id = sa.id
        WHERE sat.id = $1
        LIMIT 1
            "#,
            tx_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(res)
    }

    #[tracing::instrument]
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

    #[tracing::instrument]
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

    #[tracing::instrument]
    pub async fn set_name(
        account_id: AccountID,
        name: Option<String>,
        pool: &PgPool,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
        UPDATE simplefin_accounts
        SET custom_name = $2
        WHERE id = $1
            "#,
            account_id,
            name,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    #[tracing::instrument]
    pub fn render_edit(&self) -> maud::Markup {
        maud::html!( {
            div #{"account-" (self.id)}{
            (self.id)
                {
                (render_account_name_get_edit_form_markup(self))
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

pub struct TotalBalance {
    pub balance: Option<sqlx::postgres::types::PgMoney>,
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

    pub async fn by_date(
        aid: AccountID,
        tfo: &crate::TransactionsFilterOptions,
        pool: &PgPool,
    ) -> anyhow::Result<Vec<crate::tx::SFAccountTXGroupedQueryResultRow>> {
        let res = sqlx::query_as!(
            crate::tx::SFAccountTXGroupedQueryResultRow,
            r#"
            SELECT
                DATE_TRUNC('day', ts) as interval,
                balance as amount
            FROM simplefin_account_balances sab
            WHERE account_id = $1
            AND ($2::timestamptz IS NULL OR sab.ts >= $2)
            AND ($3::timestamptz IS NULL OR sab.ts < $3)
            "#,
            aid,
            tfo.start_datetime,
            tfo.end_datetime,
        )
        .fetch_all(pool)
        .await?;

        Ok(res)
    }

    pub async fn active_sum(pool: &PgPool) -> anyhow::Result<TotalBalance> {
        let res = sqlx::query_as!(
            TotalBalance,
            r#"
        SELECT SUM(balance) balance
        FROM simplefin_accounts sa
        JOIN (
                SELECT account_id, max(ts) as ts
                FROM simplefin_account_balances
                GROUP BY (account_id)
            ) as last_ts
            ON sa.id = last_ts.account_id
        LEFT JOIN simplefin_account_balances sab
            ON last_ts.account_id = sab.account_id
            AND last_ts.ts = sab.ts
        WHERE sa.active = true
            "#,
        )
        .fetch_one(pool)
        .await?;

        Ok(res)
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct BalancesFilter {
    active: Option<bool>,
}

pub async fn get_balances_f(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
    balances_filter: Query<BalancesFilter>,
) -> Result<Response, AppError> {
    let balances = match balances_filter.active {
        Some(true) => SFAccountBalanceQueryResult::get_active_balances(&app_state.db).await?,
        Some(false) => SFAccountBalanceQueryResult::get_inactive_balances(&app_state.db).await?,
        None => SFAccountBalanceQueryResult::get_balances(&app_state.db).await?,
    };
    let resp = maud::html! {
        (crate::html::render_balances(balances))

    };
    Ok(resp.into_response())
}

pub async fn get_balance_total_f(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
) -> Result<Response, AppError> {
    let maybe_balance = SFAccountBalance::active_sum(&app_state.db).await?;
    let resp = if let Some(balance) = maybe_balance.balance {
        maud::html! {
            (balance.to_decimal(2))
        }
    } else {
        maud::html! {
            "Cannot get balance"
        }
    };
    Ok(resp.into_response())
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

fn render_account_name_get_edit_form_markup(account: &Account) -> maud::Markup {
    maud::html! {
       div
           hx-target="this"
           hx-get={"/f/accounts/" (account.id) "/name"}
           hx-swap="outerHTML"
           hx-trigger="click"

       {
           (account.name)
           @if let Some(custom_name) = account.custom_name.clone() {
               (custom_name)
           }
           (crate::svg_icon::pencil_square())
       }
    }
}

fn render_account_name_edit(maybe_account: Option<&Account>) -> Result<Response, AppError> {
    if let Some(account) = maybe_account {
        Ok(maud::html! {
            (render_account_name_get_edit_form_markup(account))
        }
        .into_response())
    } else {
        Ok((http::StatusCode::NOT_FOUND, format!("Not Found")).into_response())
    }
}

fn render_account_name_edit_form_markup(account: &Account) -> maud::Markup {
    maud::html! {
        form
            hx-target="this"
            hx-swap="outerHTML"
            hx-post={"/f/accounts/" (account.id) "/name"}

        {
            (account.name)

            @if let Some(custom_name) = account.custom_name.clone() {
            input
                name="name"
                placeholder={(custom_name)}
                type="text"
                    {}
            } @else{
            input
                name="name"
                placeholder="New Name"
                type="text"
                    {}
            }

        }
    }
}
fn render_account_name_editable(maybe_account: Option<&Account>) -> Result<Response, AppError> {
    if let Some(account) = maybe_account {
        Ok(maud::html! {
            (render_account_name_edit_form_markup(account))

        }
        .into_response())
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

#[derive(Clone, Debug, serde::Deserialize)]
pub struct PostAccountName {
    name: String,
}

pub async fn handle_name(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
    Path(params): Path<AccountIDFilter>,
) -> Result<Response, AppError> {
    let account_id = params.account_id;
    Account::enable_sync(account_id, &app_state.db).await?;
    let maybe_account = Account::get(account_id, &app_state.db).await?;
    render_account_name_editable(maybe_account.as_ref())
}

pub async fn handle_name_post(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
    Path(params): Path<AccountIDFilter>,
    Form(form): Form<PostAccountName>,
) -> Result<Response, AppError> {
    let account_id = params.account_id;
    let mut new_name = None;
    if form.name != "" {
        new_name = Some(form.name)
    }
    Account::set_name(account_id, new_name, &app_state.db).await?;
    let maybe_account = Account::get(account_id, &app_state.db).await?;
    render_account_name_edit(maybe_account.as_ref())
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
    let balances_f = SFAccountBalanceQueryResult::get_active_balances(&app_state.db);
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
