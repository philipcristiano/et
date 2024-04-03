use clap::Parser;
use futures::try_join;
use serde::Deserialize;
use std::fs;
use std::str;

use axum::{
    extract::{FromRef, Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Form, Router,
};
use std::net::SocketAddr;

use maud::html;
use tower_cookies::CookieManagerLayer;

mod html;
mod simplefin_api;
use rust_embed::RustEmbed;

#[derive(RustEmbed, Clone)]
#[folder = "static/"]
struct StaticAssets;

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long, default_value = "127.0.0.1:3002")]
    bind_addr: String,
    #[arg(short, long, default_value = "et.toml")]
    config_file: String,
    #[arg(short, long, value_enum, default_value = "DEBUG")]
    log_level: tracing::Level,
    #[arg(long, action)]
    log_json: bool,
}

#[derive(Clone, Debug, Deserialize)]
struct AppConfig {
    database_url: String,
    auth: service_conventions::oidc::OIDCConfig,
}

#[derive(FromRef, Clone, Debug)]
struct AppState {
    auth: service_conventions::oidc::AuthConfig,
    db: PgPool,
    #[from_ref(skip)]
    db_spike: PgPool,
}

impl AppState {
    fn from_config(item: AppConfig, db: PgPool, db_spike: PgPool) -> Self {
        let auth_config = service_conventions::oidc::AuthConfig {
            oidc_config: item.auth,
            post_auth_path: "/logged_in".to_string(),
            scopes: vec!["profile".to_string(), "email".to_string()],
        };
        AppState {
            auth: auth_config,
            db,
            db_spike,
        }
    }
}

use sqlx::postgres::PgPool;
use sqlx::postgres::PgPoolOptions;

#[derive(Debug)]
pub struct SFConnection {
    id: uuid::Uuid,
    access_url: String,
}

impl SFConnection {
    #[tracing::instrument(skip_all)]
    async fn ensure_in_db(&self, pool: &PgPool) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
    INSERT INTO simplefin_connections ( id, access_url )
    VALUES ( $1, $2 )
    ON CONFLICT (id) DO NOTHING
            "#,
            self.id,
            self.access_url,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    #[tracing::instrument]
    async fn connections(
        pool: &PgPool,
    ) -> anyhow::Result<Vec<SFConnection>> {
        let res = sqlx::query_as!(
            SFConnection,
            r#"
        SELECT * FROM simplefin_connections
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(res)
    }
    #[tracing::instrument]
    async fn by_id(id: &uuid::Uuid, db: &PgPool) -> anyhow::Result<Option<SFConnection>> {
        Ok(sqlx::query_as!(
            SFConnection,
            "select * from simplefin_connections where id = $1;",
            id
        )
        .fetch_optional(db)
        .await?)
    }
}

#[derive(Debug)]
pub struct SFAccount {
    id: String,
    connection_id: uuid::Uuid,
    currency: String,
    name: String,
}
impl SFAccount {

    #[tracing::instrument]
    async fn ensure_in_db(&self, pool: &PgPool) -> anyhow::Result<()> {
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
    name: String,
    balance: sqlx::postgres::types::PgMoney,
}
impl SFAccountBalanceQueryResult {

    #[tracing::instrument]
    async fn get_balances(
        pool: &PgPool,
    ) -> anyhow::Result<Vec<SFAccountBalanceQueryResult>> {
        let res = sqlx::query_as!(
            SFAccountBalanceQueryResult,
            r#"
        SELECT name, sab.balance
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

pub struct SFAccountTXQueryResult {
    posted: chrono::NaiveDateTime,
    description: String,
    amount: sqlx::postgres::types::PgMoney,
}
impl SFAccountTXQueryResult {
    #[tracing::instrument]
    async fn for_user_id(
        user_id: &String,
        pool: &PgPool,
    ) -> anyhow::Result<Vec<SFAccountTXQueryResult>> {
        let res = sqlx::query_as!(
            SFAccountTXQueryResult,
            r#"
        SELECT sat.posted, sat.amount, sat.description
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

        Ok(res)
    }
}

#[derive(Debug)]
pub struct SFAccountBalance {
    account_id: String,
    connection_id: uuid::Uuid,
    timestamp: chrono::DateTime<chrono::Utc>,
    balance: rust_decimal::Decimal,
}
impl SFAccountBalance {
    #[tracing::instrument]
    async fn ensure_in_db(&self, pool: &PgPool) -> anyhow::Result<()> {
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
    async fn ensure_in_db(self, pool: &PgPool) -> anyhow::Result<()> {
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

    fn from_transaction(act: &SFAccount, tx: &simplefin_api::Transaction) -> Self {
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

#[derive(Clone, Debug)]
pub struct ETUser {
    pub id: String,
    pub name: String,
}

impl ETUser {
    #[tracing::instrument]
    async fn ensure_in_db(&self, pool: &PgPool) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
    INSERT INTO et_user ( id, name )
        VALUES ( $1, $2 )
    ON CONFLICT (id) DO UPDATE
        SET name = EXCLUDED.name;
            "#,
            self.id,
            self.name,
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
impl From<service_conventions::oidc::OIDCUser> for ETUser {
    fn from(item: service_conventions::oidc::OIDCUser) -> Self {
        ETUser {
            id: item.id,
            name: item.name.unwrap_or("".to_string()),
        }
    }
}

use tower_http::trace::{self, TraceLayer};
use tracing::Level;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    service_conventions::tracing::setup(args.log_level);

    let app_config = read_app_config(args.config_file);

    // Start by making a database connection.
    tracing::info!("connecting to database");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&app_config.database_url)
        .await
        .expect("Cannot connect to DB");
    let pool_spike = PgPoolOptions::new()
        .max_connections(5)
        .connect(&app_config.database_url)
        .await
        .expect("Cannot connect to DB");

    // tracing::info!("Syncing SimpleFin connections to Database");
    // for (sfconnect_id, _sfconnection_config) in app_config.simplefin.iter() {
    //     let sfc = SFConnection::new(sfconnect_id.to_string());
    //     println!("{sfc:?}");
    //     sfc.ensure_in_db(&pool).await?;

    // };

    let app_state = AppState::from_config(app_config, pool, pool_spike);
    let oidc_router = service_conventions::oidc::router(app_state.auth.clone());
    let serve_assets = axum_embed::ServeEmbed::<StaticAssets>::new();
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/logged_in", get(handle_logged_in))
        .route("/simplefin-connection/add", post(add_simplefin_connection))
        .route(
            "/simplefin-connection/:simplefin_connection_id/sync",
            post(sync_simplefin_connection),
        )
        .nest("/oidc", oidc_router.with_state(app_state.auth.clone()))
        .nest_service("/static", serve_assets)
        .with_state(app_state.clone())
        .layer(CookieManagerLayer::new())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)))
        .route("/_health", get(health));

    let addr: SocketAddr = args.bind_addr.parse().expect("Expected bind addr");
    tracing::info!("listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn read_app_config(path: String) -> AppConfig {
    let config_file_error_msg = format!("Could not read config file {}", path);
    let config_file_contents = fs::read_to_string(path).expect(&config_file_error_msg);
    let app_config: AppConfig =
        toml::from_str(&config_file_contents).expect("Problems parsing config file");

    app_config
}

async fn health() -> Response {
    "OK".into_response()
}

async fn root(
    State(app_state): State<AppState>,
    user: Option<service_conventions::oidc::OIDCUser>,
) -> Result<Response, AppError> {
    if let Some(user) = user {
        let user_connections_f = SFConnection::connections(&app_state.db);
        let balances_f = SFAccountBalanceQueryResult::get_balances(&app_state.db);
        let transactions_f = SFAccountTXQueryResult::for_user_id(&user.id, &app_state.db);

        let (user_connections, balances, transactions) =
            try_join!(user_connections_f, balances_f, transactions_f)?;

        Ok(html::maud_page(html! {
              p { "Welcome! " ( user.id)}
              @if let Some(name) = user.name {
                  p{ ( name ) }
              }
              @if let Some(email) = user.email {
                  p{ ( email ) }
              }

              h2 { "Connections:" }
              @for sfconn in &user_connections {
              div {
                  form method="post" action={"/simplefin-connection/" (sfconn.id) "/sync"} {
                    (sfconn.id)
                    input type="submit" class="border" value="Sync connection" {}
                  }
              }}

              div {
                h3 { "Add a SimpleFin Connection"}
                form method="post" action="/simplefin-connection/add" {
                  input id="simplefin_token" class="border min-w-full" name="simplefin_token" {}
                  input type="submit" class="border" {}
              }
              }

              h2 { "Accounts:" }
              table class="table-auto"{
                  thead {
                    tr {
                        th { "Account"}
                        th { "Balance"}
                    }
                  }
                  tbody {
                  @for balance in &balances {
                  tr{
                    td { (balance.name)}
                    td { (balance.balance.to_decimal(2))}
                  }
                  }
                  }

              }

              h2 { "Transactions:" }
              table class="table-auto"{
                  thead {
                    tr {
                        th { "Date"}
                        th { "Description"}
                        th { "Amount"}
                    }
                  }
                  tbody {
                  @for tx in &transactions {
                  tr{
                    td { (tx.posted)}
                    td { (tx.description)}
                    td { (tx.amount.to_decimal(2))}
                  }
                  }
                  }

              }



        })
        .into_response())
    } else {
        Ok(html::maud_page(html! {
            p { "Welcome! You need to login" }
            a href="/oidc/login" { "Login" }
        })
        .into_response())
    }
}

async fn handle_logged_in(
    State(app_state): State<AppState>,
    user: service_conventions::oidc::OIDCUser,
) -> Result<Response, AppError> {
    let et_user = ETUser::from(user);
    et_user.ensure_in_db(&app_state.db).await?;
    Ok(Redirect::to("/").into_response())
}

#[derive(Clone, Debug, Deserialize)]
struct PostSimplefinToken {
    simplefin_token: String,
}

use uuid;
async fn add_simplefin_connection(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
    Form(form): Form<PostSimplefinToken>,
) -> Result<Response, AppError> {
    let access_url = simplefin_api::token_to_access_url(form.simplefin_token).await?;

    tracing::info!("access_url to {}", access_url);

    let id = uuid::Uuid::new_v4();
    let sfc = SFConnection {
        id,
        access_url,
    };
    tracing::info!("saving access_url");
    sfc.ensure_in_db(&app_state.db).await?;

    Ok(Redirect::to("/").into_response())
}

#[derive(Deserialize)]
struct RequestSyncParams {
    simplefin_connection_id: String,
}
async fn sync_simplefin_connection(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
    Path(RequestSyncParams {
        simplefin_connection_id,
    }): Path<RequestSyncParams>,
) -> Result<Response, AppError> {
    tracing::info!("Request Sync for {}", simplefin_connection_id);
    let uuid_id = uuid::Uuid::parse_str(&simplefin_connection_id)?;
    let sfc = SFConnection::by_id(&uuid_id, &app_state.db).await?;
    if let Some(sfc) = sfc {
        let account_set = simplefin_api::accounts(&sfc.access_url).await?;
        for account in account_set.accounts {
            tracing::debug!("Saving account: {:?}", account.id);
            let sfa = SFAccount {
                id: account.id,
                connection_id: sfc.id,
                name: account.name,
                currency: account.currency,
            };
            sfa.ensure_in_db(&app_state.db).await?;
            let sfab = SFAccountBalance {
                account_id: sfa.id.clone(),
                connection_id: sfc.id,
                timestamp: account.balance_date,
                balance: account.balance,
            };
            sfab.ensure_in_db(&app_state.db).await?;

            let txs_f = account.transactions.iter().map(|src_tx| {
                    let tx = SFAccountTransaction::from_transaction(&sfa, &src_tx);
                    SFAccountTransaction::ensure_in_db(tx, &app_state.db_spike)
                });

            futures::future::try_join_all(txs_f).await?;
            ()

        }
    }
    Ok(Redirect::to("/").into_response())
}
// Make our own error that wraps `anyhow::Error`.
struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
