use sqlx::postgres::PgPool;

use futures::try_join;

use axum::{
    extract::{FromRef, Path, Query, State},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Form,
};

use crate::{html, AppState, SFConnection};

pub async fn handle_labels(
    State(app_state): State<AppState>,
    user: service_conventions::oidc::OIDCUser,
) -> Result<Response, crate::AppError> {
    let user_connections_f = SFConnection::connections(&app_state.db);
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
          div #main class="main" {
            div {
              h3 { "Add a label"}
              form method="post" action="/labels" {
                input id="label" class="border min-w-full" name="label" {}
                input type="submit" class="border" {}
              }
            }
            (&labels_result)
          }

    }
    .into_response())
}

#[derive(sqlx::FromRow)]
struct Label {
    id: uuid::Uuid,
    label: sqlx::postgres::types::PgLTree,
}

use std::str::FromStr;
impl Label {
    fn new(l: String) -> anyhow::Result<Label> {
        let id = uuid::Uuid::new_v4();
        let label = sqlx::postgres::types::PgLTree::from_str(&l)?;
        Ok(Label { id, label })
    }

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

struct LabelsQuery {
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
}

impl maud::Render for LabelsQuery {
    fn render(&self) -> maud::Markup {
        maud::html! {
           h2 { "Labels:" }
           table #transaction-table class="table-auto"{
               thead {
                 tr {
                     th { "Label"}
                 }
               }
               tbody {
               @for label in &self.item {
               tr{
                    td {(label.label)}
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
