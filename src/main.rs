use clap::Parser;
use futures::try_join;
use serde::{Deserialize, Serialize};
use std::fs;
use std::ops::Deref;
use std::str;

use axum::{
    extract::{FromRef, Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::{delete, get, post},
    Form, Router,
};
use axum_extra::extract::Query;
use std::net::SocketAddr;

use maud::html;
use tower_cookies::CookieManagerLayer;

mod accounts;
mod charts;
mod dates;
mod html;
mod labels;
mod rules;
mod simplefin_api;
mod svg_icon;
mod sync_manager;
mod tx;
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
    #[serde(default)]
    features: AppFeatures,
}

#[derive(Clone, Debug, Deserialize)]
struct AppFeatures {
    #[serde(default)]
    charts: bool,
    #[serde(default)]
    apply_rules: bool,
}

impl Default for AppFeatures {
    fn default() -> Self {
        AppFeatures {
            charts: false,
            apply_rules: false,
        }
    }
}

#[derive(FromRef, Clone, Debug)]
struct AppState {
    auth: service_conventions::oidc::AuthConfig,
    db: PgPool,
    #[from_ref(skip)]
    db_spike: PgPool,
    features: AppFeatures,
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
            features: item.features,
        }
    }
}

use sqlx::postgres::PgPool;
use sqlx::postgres::PgPoolOptions;

#[derive(Clone, Debug, Copy, PartialEq, Deserialize, sqlx::Type)]
#[sqlx(transparent)]
pub struct ConnectionID(uuid::Uuid);
impl std::fmt::Display for ConnectionID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "ConnectionID: {}", self.0)
    }
}
impl From<uuid::Uuid> for ConnectionID {
    fn from(item: uuid::Uuid) -> Self {
        ConnectionID(item)
    }
}

impl ConnectionID {
    fn new() -> Self {
        ConnectionID(uuid::Uuid::new_v4())
    }

    fn to_uuid(&self) -> &uuid::Uuid {
        &self.0
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Connection {
    id: ConnectionID,
    access_url: String,
}
#[derive(Debug, Clone)]
pub struct SFConnectionSyncInfo {
    connection_id: ConnectionID,
    ts: chrono::DateTime<chrono::Utc>,
}

impl SFConnectionSyncInfo {
    fn is_since(&self, since: chrono::Duration) -> bool {
        let now = chrono::Utc::now();
        let diff = now - self.ts;
        let ret = since > diff;
        let ts = &self.ts;
        tracing::debug!("Comparing times now {now:?}, ts {ts:?} diff {diff:?}, ret {ret}");
        ret
    }
}
#[derive(Clone, Debug, Deserialize)]
pub struct ConnectionSyncError {
    connection_id: ConnectionID,
    ts: Option<chrono::DateTime<chrono::Utc>>,
    message: String,
}

impl Connection {
    #[tracing::instrument(skip_all)]
    async fn ensure_in_db(&self, pool: &PgPool) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
    INSERT INTO simplefin_connections ( id, access_url )
    VALUES ( $1, $2 )
    ON CONFLICT (id) DO NOTHING
            "#,
            self.id.0,
            self.access_url,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    #[tracing::instrument]
    async fn connections(pool: &PgPool) -> anyhow::Result<Vec<Connection>> {
        let res = sqlx::query_as!(
            Connection,
            r#"
        SELECT * FROM simplefin_connections
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(res)
    }
    #[tracing::instrument]
    async fn last_sync_info(&self, pool: &PgPool) -> anyhow::Result<Option<SFConnectionSyncInfo>> {
        let res = sqlx::query_as!(
            SFConnectionSyncInfo,
            r#"
        SELECT connection_id, ts FROM simplefin_connection_sync_info
        WHERE connection_id = $1
        ORDER BY ts DESC
        LIMIT 1;
            "#,
            self.id.0
        )
        .fetch_optional(pool)
        .await?;
        Ok(res)
    }
    #[tracing::instrument]
    async fn mark_synced(&self, errors: &Vec<String>, pool: &PgPool) -> anyhow::Result<()> {
        let now = chrono::Utc::now();
        sqlx::query!(
            r#"
    INSERT INTO simplefin_connection_sync_info ( connection_id, ts )
    VALUES ( $1, $2 )
    ON CONFLICT (connection_id, ts) DO NOTHING
            "#,
            self.id.0,
            now,
        )
        .execute(pool)
        .await?;
        for error in errors {
            self.save_error(&now, &error, pool).await?;
        }

        Ok(())
    }

    #[tracing::instrument]
    async fn save_error(
        &self,
        now: &chrono::DateTime<chrono::Utc>,
        error: &String,
        pool: &PgPool,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
    INSERT INTO simplefin_connection_sync_errors ( connection_id, ts, message )
    VALUES ( $1, $2, $3 )
    ON CONFLICT (connection_id, ts) DO NOTHING
            "#,
            self.id.0,
            now,
            error,
        )
        .execute(pool)
        .await?;
        Ok(())
    }

    async fn get_last_sync_errors(
        connection_id: ConnectionID,
        pool: &PgPool,
    ) -> anyhow::Result<Vec<ConnectionSyncError>> {
        let res = sqlx::query_as!(
            ConnectionSyncError,
            r#"
        SELECT connection_id, last_sync.ts, message FROM
            ( SELECT max(ts) as ts
              FROM simplefin_connection_sync_info
              WHERE connection_id = $1 ) AS last_sync
        JOIN simplefin_connection_sync_errors AS scse
        ON scse.connection_id = $1
        AND scse.ts = last_sync.ts
            "#,
            connection_id.to_uuid()
        )
        .fetch_all(pool)
        .await?;

        Ok(res)
    }

    #[tracing::instrument]
    async fn by_id(id: &uuid::Uuid, db: &PgPool) -> anyhow::Result<Option<Connection>> {
        Ok(sqlx::query_as!(
            Connection,
            "select * from simplefin_connections where id = $1;",
            id
        )
        .fetch_optional(db)
        .await?)
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

    let app_state = AppState::from_config(app_config, pool, pool_spike);

    let app_state2 = app_state.clone();
    tokio::spawn(async move {
        sync_manager::sync_all(app_state2).await;
    });

    let oidc_router = service_conventions::oidc::router(app_state.auth.clone());
    let serve_assets = axum_embed::ServeEmbed::<StaticAssets>::new();
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/f/connection/{connection_id}", get(get_connection_f))
        .route("/accounts", get(crate::accounts::get_accounts_f))
        .route(
            "/f/accounts/{account_id}",
            get(crate::accounts::get_account_f),
        )
        .route(
            "/f/accounts/{account_id}/active",
            post(crate::accounts::handle_active_post).delete(crate::accounts::handle_active_delete),
        )
        .route(
            "/f/accounts/{account_id}/name",
            get(crate::accounts::handle_name).post(crate::accounts::handle_name_post),
        )
        .route(
            "/f/accounts/{account_id}/delete-transactions",
            post(crate::accounts::handle_delete_transactions),
        )
        .route("/chart", get(crate::charts::get_chart))
        .route("/accounts/{account_id}", get(crate::accounts::get_account))
        .route("/balances/f", get(crate::accounts::get_balances_f))
        .route(
            "/balances/f/total",
            get(crate::accounts::get_balance_total_f),
        )
        .route("/f/transactions", get(get_transactions))
        .route("/f/transactions/value", get(get_transactions_value))
        .route(
            "/f/transactions/{transaction_id}/edit",
            get(crate::tx::handle_tx_edit_get).post(crate::tx::handle_tx_edit_post),
        )
        .route(
            "/f/transaction_label",
            post(crate::tx::handle_tx_add_label).delete(crate::tx::handle_tx_delete_label),
        )
        .route(
            "/labels",
            get(labels::handle_labels).post(labels::add_label),
        )
        .route("/f/labels", get(labels::handle_labels_fragment))
        .route(
            "/f/labels/search",
            get(labels::handle_labels_search_fragment),
        )
        .route("/logged_in", get(handle_logged_in))
        .route("/simplefin-connection/add", post(add_simplefin_connection))
        .route("/rules", get(rules::handle_rules))
        .route("/f/rules", get(rules::handle_rules_fragment))
        .route("/f/rules/new", post(rules::handle_new_rule_fragment))
        .route(
            "/rules/{rule_id}",
            get(rules::handle_rule).delete(rules::handle_rule_delete),
        )
        .route(
            "/f/rules/{rule_id}/labels/search",
            get(rules::handle_labels_search_fragment),
        )
        .route("/f/rules/{rule_id}/labels", post(rules::handle_label_add))
        .route(
            "/f/rules/{rule_id}/labels/{label_id}",
            delete(rules::handle_label_delete),
        )
        .nest("/oidc", oidc_router.with_state(app_state.auth.clone()))
        .nest_service("/static", serve_assets)
        .with_state(app_state.clone())
        .layer(CookieManagerLayer::new())
        .layer(tower_http::compression::CompressionLayer::new())
        .layer(service_conventions::tracing_http::trace_layer(
            tracing::Level::INFO,
        ))
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

#[derive(serde::Deserialize, Clone, Debug)]
pub struct ConnectionIDFilter {
    pub connection_id: ConnectionID,
}
async fn get_connection_f(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
    Path(params): Path<ConnectionIDFilter>,
) -> Result<Response, AppError> {
    let connection_id = params.connection_id;
    let errors = Connection::get_last_sync_errors(connection_id, &app_state.db).await?;
    let resp = html! {
        (connection_id)
        @for e in errors {
            p { (svg_icon::exclamation_circle()) (e.message) }
        }

    };
    Ok(resp.into_response())
}

async fn get_account_f(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
    Path(params): Path<ConnectionIDFilter>,
) -> Result<Response, AppError> {
    let connection_id = params.connection_id;
    let errors = Connection::get_last_sync_errors(connection_id, &app_state.db).await?;
    let resp = html! {
        (connection_id)
        @for e in errors {
            p { (svg_icon::exclamation_circle()) (e.message) }
        }

    };
    Ok(resp.into_response())
}

pub type Label = String;
pub type DescriptionFragment = String;

#[derive(Deserialize, Debug, Clone, Default, Serialize)]
struct TransactionsFilterOptions {
    pub account_id: Option<crate::accounts::AccountID>,
    #[serde(default)]
    pub labeled: Vec<Label>,
    #[serde(default)]
    pub not_labeled: Vec<Label>,
    pub description_contains: Option<DescriptionFragment>,
    pub transaction_id: Option<crate::tx::TransactionID>,

    pub start_datetime: Option<chrono::DateTime<chrono::Utc>>,
    pub end_datetime: Option<chrono::DateTime<chrono::Utc>>,
}

impl TransactionsFilterOptions {
    fn to_querystring(&self) -> Result<String, serde_html_form::ser::Error> {
        let qs = serde_html_form::to_string(&self)?;
        tracing::debug!(qs = qs, tfo= ?self, "To querystring");
        Ok(qs)
    }

    fn with_transaction_id(
        &self,
        t: crate::tx::TransactionID,
    ) -> anyhow::Result<TransactionsFilterOptions> {
        let new = TransactionsFilterOptions {
            transaction_id: Some(t),
            ..self.clone()
        };

        Ok(new)
    }

    pub fn from_querystring(qs: &str) -> anyhow::Result<Self> {
        let tfo: Self = serde_html_form::from_str(qs)?;
        Ok(tfo)
    }

    fn with_pltree(
        &self,
        t: sqlx::postgres::types::PgLTree,
    ) -> anyhow::Result<TransactionsFilterOptions> {
        let mut new_labeled = self.labeled.clone();
        new_labeled.push(t.to_string());
        let new = TransactionsFilterOptions {
            labeled: new_labeled,
            ..self.clone()
        };

        Ok(new)
    }

    fn without_pltree(
        &self,
        t: sqlx::postgres::types::PgLTree,
    ) -> anyhow::Result<TransactionsFilterOptions> {
        self.without_label(t.to_string())
    }

    fn with_label(&self, label: String) -> anyhow::Result<TransactionsFilterOptions> {
        let mut new_labeled = self.labeled.clone();
        new_labeled.push(label);
        let new = TransactionsFilterOptions {
            labeled: new_labeled,
            ..self.clone()
        };

        Ok(new)
    }

    fn without_label(&self, label: String) -> anyhow::Result<TransactionsFilterOptions> {
        let mut new_not_labeled = self.not_labeled.clone();
        new_not_labeled.push(label);
        let new = TransactionsFilterOptions {
            not_labeled: new_not_labeled,
            ..self.clone()
        };

        Ok(new)
    }

    fn with_datetimes(
        &self,
        start: Option<chrono::DateTime<chrono::Utc>>,
        end: Option<chrono::DateTime<chrono::Utc>>,
    ) -> TransactionsFilterOptions {
        let new = TransactionsFilterOptions {
            start_datetime: start,
            end_datetime: end,
            ..self.clone()
        };

        new
    }

    pub fn render_to_hidden_input_fields(self) -> maud::Markup {
        let options: TransactionsFilterOptions = self.into();
        maud::html! {

            @if let Some(txid) = options.transaction_id {
                input type="hidden" name="transaction_id" value={(txid)} {}
            }
            @if let Some(account_id) = options.account_id {
                input type="hidden" name="account_id" value={(account_id)} {}
            }
            @if let Some(fragment) = options.description_contains {
                input type="hidden" name="description_contains" value={(fragment)} {}
            }
            @for as_labeled in options.labeled {
                input type="hidden" name="labeled" value={(as_labeled)} {}
            }
            @for not_labeled in options.not_labeled {
                input type="hidden" name="not_labeled" value={(not_labeled)} {}
            }
            @if let Some(start_datetime) = options.start_datetime {
                input type="hidden" name="start_datetime" value={(start_datetime)} {}
            }
        }
    }
}

use maud::Render;
async fn get_transactions(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
    tx_filter: Query<TransactionsFilterOptions>,
) -> Result<Response, AppError> {
    let filter_options = tx_filter.deref();
    //let transactions =
    //tx::SFAccountTXQuery::from_options(filter_options.clone().into(), &app_state.db).await?;

    let transactions =
        tx::SFAccountTXQuery::from_filter_options(filter_options, &app_state.db).await?;
    let qs = filter_options.to_querystring()?;

    Ok(maud::html!(
                @if app_state.features.charts{
                div
                        hx-target="this"
                        hx-swap="innerHTML"
                        hx-get={"/chart?" (qs) "&x_size=720&y_size=240" }
                        hx-trigger="load" {}
                }
                (transactions.render())
    )
    .into_response())
}

async fn get_transactions_value(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
    tx_filter: Query<TransactionsFilterOptions>,
) -> Result<Response, AppError> {
    let filter_options = tx_filter.deref();
    let value =
        tx::SFAccountTXQuery::amount_from_filter_options(&filter_options, &app_state.db).await?;

    let r = maud::html! {
        a
            href={"/?" (filter_options.to_querystring()?) }
            {(value.render())}
    };
    Ok(r.into_response())
}

async fn root(
    State(app_state): State<AppState>,
    user: Result<
        Option<service_conventions::oidc::OIDCUser>,
        service_conventions::oidc::OIDCUserError,
    >,
    tx_filter: Query<TransactionsFilterOptions>,
) -> Result<Response, AppError> {
    if let Ok(Some(_user)) = user {
        let filter_options = tx_filter.deref().clone();
        tracing::debug!("Transaction Filter Options {:?}", &filter_options);
        let user_connections_f = Connection::connections(&app_state.db);
        let balances_f = accounts::SFAccountBalanceQueryResult::get_active_balances(&app_state.db);
        //let transactions_f =
        //   tx::SFAccountTXQuery::from_options(filter_options.into(), &app_state.db);

        let transactions_f =
            tx::SFAccountTXQuery::from_filter_options(&filter_options, &app_state.db);

        let (user_connections, balances, transactions) =
            try_join!(user_connections_f, balances_f, transactions_f)?;

        let qs = filter_options.to_querystring()?;
        Ok(html::maud_page(html! {
              div class="flex flex-col lg:flex-row"{
              (html::sidebar(user_connections, balances))
              div #main class="main" {
                // #TODO: add in tx filtering
                // #TODO: Fix description_contains in qs and separate input field

                form
                      hx-get={"/f/transactions?" (qs)}
                      hx-push-url={"/?" (qs) }
                      hx-target={"#transaction-list"}
    o                 hx-trigger="input changed delay:100ms from:input#search-input-transactions"
                 {
                  input #{ "search-input-transactions"}
                      name="description_contains"
                      placeholder="Begin typing to search transactions"
                  {}

                }

                @if app_state.features.charts{
                div
                        hx-target="this"
                        hx-swap="innerHTML"
                        hx-get={"/chart?" (qs) "&x_size=720&y_size=240" }
                        hx-trigger="load" {}
                }
                div #save-as-rule {
                    (rules::new_from_filter_options_html_box(filter_options.clone())?)
                }
                div #transaction-list {
                    (tx::label_search_box(&("root_tx".to_string()), filter_options.clone())?)
                    (&transactions)}
              }}

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

    let id = ConnectionID::new();
    let sfc = Connection { id, access_url };
    tracing::info!("saving access_url");
    sfc.ensure_in_db(&app_state.db).await?;

    Ok(Redirect::to("/").into_response())
}

// Make our own error that wraps `anyhow::Error`.
#[derive(Debug)]
struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        tracing::error!("HTTP Error {:?}", &self);
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

#[cfg(test)]
mod test_transactions_filter_options {
    use super::*;

    #[test]
    fn test_query_string() {
        let tfo = TransactionsFilterOptions::default();
        assert_eq!(tfo.to_querystring().unwrap(), "")
    }
    #[test]
    fn test_query_string_label_list() {
        let tfo = TransactionsFilterOptions::default()
            .with_label("label1".to_string())
            .unwrap();
        assert_eq!(tfo.to_querystring().unwrap(), "labeled=label1")
    }
}
