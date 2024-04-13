use sqlx::postgres::PgPool;

use futures::try_join;
use crate::svg_icon;

use axum::{
    extract::{FromRef, Path, Query, State},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Form,
};

use crate::{html, AppState, Connection};

pub async fn handle_labels(
    State(app_state): State<AppState>,
    user: service_conventions::oidc::OIDCUser,
) -> Result<Response, crate::AppError> {
    let user_connections_f = Connection::connections(&app_state.db);
    let balances_f = crate::accounts::SFAccountBalanceQueryResult::get_balances(&app_state.db);
    let labels_fut = LabelsQuery::all(&app_state.db);

    let (user_connections, balances, labels_result) =
        try_join!(user_connections_f, balances_f, labels_fut)?;

    Ok(html::maud_page(html! {
          div class="flex flex-col lg:flex-row"{
          (html::sidebar(user_connections, balances))
          div #main class="main" {

            div {
              h3 { "Add a label"}
              form method="post" action="/labels" {
                input id="label" class="border min-w-full" name="label" {}
                input type="submit" class="border" {}
              }
            }
            (&labels_result)
          }}

    })
    .into_response())
}
pub async fn handle_labels_fragment(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
) -> Result<Response, crate::AppError> {
    let labels_result = LabelsQuery::all(&app_state.db).await?;

    Ok(html! {
        div {
          h3 { "Add a label"}
          form method="post" action="/labels" {
            input id="label" class="border min-w-full" name="label" {}
            input type="submit" class="border" {}
          }
        }
        (&labels_result)

    }
    .into_response())
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct LabelSearch {
    search: String,
    #[serde(flatten)]
    full_transaction_id: crate::tx::FullTransactionID,
}

pub async fn handle_labels_search_fragment(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
    Form(form): Form<LabelSearch>,
) -> Result<Response, crate::AppError> {
    let results = LabelsQuery::search(form.search, &app_state.db).await?;
    Ok(
        html! {(results.render_add_labels_for_tx(form.full_transaction_id.clone()))}
            .into_response(),
    )
}

pub type LabelID = uuid::Uuid;
#[derive(sqlx::FromRow, Debug)]
struct Label {
    id: LabelID,
    label: sqlx::postgres::types::PgLTree,
}

use std::str::FromStr;
impl Label {
    fn new(l: String) -> anyhow::Result<Label> {
        let id = uuid::Uuid::new_v4();
        let label = sqlx::postgres::types::PgLTree::from_str(&l)?;
        Ok(Label { id, label })
    }

    #[tracing::instrument]
    async fn ensure_in_db(&self, pool: &PgPool) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
    INSERT INTO labels ( id, label )
    VALUES ( $1, $2 )
    ON CONFLICT (id) DO NOTHING
            "#,
            self.id,
            self.label,
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

pub struct LabelsQuery {
    item: Vec<Label>,
}

impl From<Vec<Label>> for LabelsQuery {
    fn from(item: Vec<Label>) -> Self {
        LabelsQuery { item }
    }
}

impl LabelsQuery {
    #[tracing::instrument]
    pub async fn all(pool: &PgPool) -> anyhow::Result<Self> {
        let res = sqlx::query_as!(
            Label,
            r#"
        SELECT id, label
        FROM labels l
        ORDER BY
            l.label ASC
            "#,
        )
        .fetch_all(pool)
        .await?;
        Ok(res.into())
    }

    #[tracing::instrument]
    pub async fn search(name: String, pool: &PgPool) -> anyhow::Result<Self> {
        tracing::debug!("Label name {:?}", &name);
        let query_label = sqlx::postgres::types::PgLQueryLevel::from_str(&format!("{name}*"))?;
        let star = sqlx::postgres::types::PgLQueryLevel::from_str("*")?;
        tracing::debug!("Label  {:?}", &query_label);
        let qv = vec![star.clone(), query_label, star.clone()];
        let query = sqlx::postgres::types::PgLQuery::from(qv);
        tracing::debug!("Label query {:?}", &query);

        let res = sqlx::query_as!(
            Label,
            r#"
        SELECT id, label
        FROM labels l
        WHERE label ~ $1
        ORDER BY
            l.label ASC
            "#,
            query as _,
        )
        .fetch_all(pool)
        .await?;

        Ok(res.into())
    }

    #[tracing::instrument]
    pub async fn for_tx(
        ftxid: &crate::tx::FullTransactionID,
        pool: &PgPool,
    ) -> anyhow::Result<Self> {
        let res = sqlx::query_as!(
            Label,
            r#"
        SELECT id, label
        FROM labels l
        JOIN transaction_labels tl
            ON l.id = tl.label_id
        WHERE tl.transaction_id = $1
        ORDER BY
            l.label ASC
            "#,
            ftxid.transaction_id
        )
        .fetch_all(pool)
        .await?;

        Ok(res.into())
    }

    pub fn render_as_table_for_tx(&self, ftxid: crate::tx::FullTransactionID) -> maud::Markup {
        maud::html! {
           table
               #{"transaction-labels-" (ftxid.transaction_id)}
               class="table-auto"{
               tbody {
               @for label in &self.item {
               tr{
                    td {
                        form
                            hx-target={"#transaction-labels-" (ftxid.transaction_id)}
                            hx-delete={"/f/transaction_label"}
                            hx-trigger="click"
                        {

                            input type="hidden" name="label_id" value={(label.id)} {}
                            input type="hidden" name="transaction_id" value={(ftxid.transaction_id)} {}
                            (crate::svg_icon::x_circle())
                        }

                             (label.label)

                       }
                    }
                 }
               }

           }
        }
    }

    fn render_add_labels_for_tx(&self, ftxid: crate::tx::FullTransactionID) -> maud::Markup {
        maud::html! {
           table class="table-auto"{
               thead {
                 tr {
                     th { "Label"}
                 }
               }
               tbody {
               @for label in &self.item {
               tr{
                    td{

                        form
                        hx-target={"#transaction-labels-" (ftxid.transaction_id)}
                        hx-post={"/f/transaction_label"}
                        hx-trigger="click"
                        {

                            input type="hidden" name="label_id" value={(label.id)} {}
                            input type="hidden" name="transaction_id" value={(ftxid.transaction_id)} {}
                            {
                             (label.label)
                            }
                       }
                 }}
               }
               }

           }
        }
    }
}

impl maud::Render for LabelsQuery {
    fn render(&self) -> maud::Markup {
        maud::html! {
           table #labels-table class="table-auto"{
               tbody {
               @for label in &self.item {
               tr{
                    td { (label.label) }
                    td
                        hx-get={"/f/transactions?not_labeled=" (label.label) }
                        hx-push-url={"/?not_labeled=" (label.label) }
                        hx-target="#main"
                        hx-swap="innerHTML"
                        hx-trigger="click"
                        {
                            (svg_icon::magnifying_glass_minus())
                        }

                    td
                        hx-get={"/f/transactions?labeled=" (label.label) }
                        hx-push-url={"/?labeled=" (label.label) }
                        hx-target="#main"
                        hx-swap="innerHTML"
                        hx-trigger="click"
                        {
                            (svg_icon::magnifying_glass_plus())
                        }
                 }
               }
               }

           }
        }
    }
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct LabelPost {
    label: String,
}

pub async fn add_label(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
    Form(form): Form<LabelPost>,
) -> Result<Response, crate::AppError> {
    let l = Label::new(form.label)?;
    tracing::info!("saving label");
    l.ensure_in_db(&app_state.db).await?;

    Ok(Redirect::to("/labels").into_response())
}
