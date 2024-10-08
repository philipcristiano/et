use sqlx::postgres::PgPool;

use crate::svg_icon;
use futures::try_join;

use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
    Form,
};

use crate::{html, AppState, Connection};

pub async fn handle_labels(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
) -> Result<Response, crate::AppError> {
    let user_connections_f = Connection::connections(&app_state.db);
    let balances_f = crate::accounts::SFAccountBalanceQueryResult::get_balances(&app_state.db);
    let labels_fut = LabelsQuery::all(&app_state.db);

    let (user_connections, balances, labels_result) =
        try_join!(user_connections_f, balances_f, labels_fut)?;
    let f = crate::TransactionsFilterOptions::default();

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
            (&labels_result.render_with_tx_filter(f)?)
          }}

    })
    .into_response())
}
pub async fn handle_labels_fragment(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
) -> Result<Response, crate::AppError> {
    let labels_result = LabelsQuery::all(&app_state.db).await?;
    let f = crate::TransactionsFilterOptions::default();
    Ok(html! {
        div {
          h3 { "Add a label"}
          form method="post" action="/labels" {
            input id="label" class="border min-w-full" name="label" {}
            input type="submit" class="border" {}
          }
        }
        (&labels_result.render_with_tx_filter(f)?)

    }
    .into_response())
}

#[derive(serde::Deserialize, Debug, Clone)]
pub struct LabelSearch {
    search: String,
    #[serde(flatten)]
    transaction_filter: crate::TransactionsFilterOptions,
}

pub async fn handle_labels_search_fragment(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
    Form(form): Form<LabelSearch>,
) -> Result<Response, crate::AppError> {
    let results = LabelsQuery::search(form.search, &app_state.db).await?;
    let tx_filter: crate::TransactionsFilterOptions = form.transaction_filter.into();
    Ok(html! {(results.render_add_labels_for_tx(tx_filter))}.into_response())
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
        let query_label = sqlx::postgres::types::PgLQueryLevel::from_str(&format!("{name}@*"))?;
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
    pub async fn for_tx(ftxid: &crate::tx::TransactionID, pool: &PgPool) -> anyhow::Result<Self> {
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
            ftxid
        )
        .fetch_all(pool)
        .await?;

        Ok(res.into())
    }

    pub fn render_as_table_for_tx(&self, ftxid: crate::tx::TransactionID) -> maud::Markup {
        maud::html! {
           table
               #{"transaction-labels-" (ftxid)}
               class="table-auto"{
               tbody {
               @for label in &self.item {
               tr{
                    td {
                        form
                            hx-target={"#transaction-labels-" (ftxid)}
                            hx-delete={"/f/transaction_label"}
                            hx-trigger="click"
                        {

                            input type="hidden" name="label_id" value={(label.id)} {}
                            input type="hidden" name="transaction_id" value={(ftxid)} {}
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

    fn render_add_labels_for_tx(&self, txf: crate::TransactionsFilterOptions) -> maud::Markup {
        let options = txf.clone().render_to_hidden_input_fields();

        let mut hx_target = String::new();
        if let Some(tid) = txf.transaction_id {
            hx_target = format!("#transaction-labels-{}", tid);
        };
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
                        hx-target={(hx_target)}
                        hx-post={"/f/transaction_label"}
                        hx-trigger="click"
                        {

                            input type="hidden" name="label_id" value={(label.id)} {}
                            (options)
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

    fn render_with_tx_filter(
        &self,
        txf: crate::TransactionsFilterOptions,
    ) -> anyhow::Result<maud::Markup> {
        let now = chrono::Utc::now();
        let ago_30 = now - chrono::Duration::days(30);
        let midnight = chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        let start_datetime_30 = ago_30.with_time(midnight).unwrap();
        let start_end_pairs = crate::dates::month_ranges(now, 4)?;
        Ok(maud::html! {
           table #labels-table class="table-auto"{

               thead {
                 tr {
                     th { "Label"}
                     th {}
                     th {}
                     th { "Last 30 days"}
                     th { "This Month"}
                     th { "Last Month"}
                     th { "Last Last Month"}
                 }
               }
               tbody {
               @for label in &self.item {
               @let label_txf = txf.with_pltree(label.label.clone())?;
               tr{
                    td { (label.label) }
                    td
                        hx-get={"/f/transactions?" (txf.without_pltree(label.label.clone())?.to_querystring()?) }
                        hx-push-url={"/?not_labeled=" (label.label) }
                        hx-target="#main"
                        hx-swap="innerHTML"
                        hx-trigger="click"
                        {
                            (svg_icon::magnifying_glass_minus())
                        }

                    td
                        hx-get={"/f/transactions?" (label_txf.clone().to_querystring()?) }
                        hx-push-url={"/?labeled=" (label.label) }
                        hx-target="#main"
                        hx-swap="innerHTML"
                        hx-trigger="click"
                        {
                            (svg_icon::magnifying_glass_plus())
                        }
                    td
                        hx-get={"/f/transactions/value?" (
                            label_txf.with_datetimes(Some(start_datetime_30), Some(now)).to_querystring()?)
                                 }
                        hx-target="this"
                        hx-swap="innerHTML"
                        hx-trigger="load"
                        {
                            "Loading..."
                        }
                    @for (s, e) in &start_end_pairs {
                    @let label_se_txf = label_txf.clone().with_datetimes(Some(s.to_owned()), Some(e.to_owned()));
                    td
                        hx-get={"/f/transactions/value?" (
                            label_se_txf.to_querystring()?)
                                 }
                        hx-target="this"
                        hx-swap="innerHTML"
                        hx-trigger="load"
                        {
                            "Loading..."
                        }
                    }
                 }
               }
               }

           }
        })
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
