use sqlx::postgres::PgPool;

use crate::svg_icon;
use crate::tx;
use crate::TransactionsFilterOptions;
use axum::http::HeaderMap;
use axum_extra::extract::{Form, Query};
use futures::try_join;
use std::ops::Deref;

use axum::{
    extract::{Path, State},
    response::{AppendHeaders, IntoResponse, Redirect, Response},
};

use crate::{html, AppState, Connection};

pub type RuleID = uuid::Uuid;

#[derive(serde::Deserialize, Debug, Clone)]
pub struct Rule {
    id: RuleID,
    name: String,
    transaction_filter_qs: String,
}

impl Rule {
    fn try_new(name: String, tf: TransactionsFilterOptions) -> anyhow::Result<Self> {
        let id = uuid::Uuid::now_v7();
        let qs = tf.to_querystring()?;
        Ok(Rule {
            id,
            name,
            transaction_filter_qs: qs,
        })
    }

    fn get_transaction_filter(&self) -> anyhow::Result<crate::TransactionsFilterOptions> {
        crate::TransactionsFilterOptions::from_querystring(self.transaction_filter_qs.as_str())
    }

    async fn get_labels(&self, pool: &PgPool) -> anyhow::Result<Vec<crate::labels::Label>> {
        let res = sqlx::query_as!(
            crate::labels::Label,
            r#"
        SELECT id, label
        FROM labels l
        JOIN rules_labels rl
            ON l.id = rl.label_id
        WHERE
            rl.rule_id = $1
            "#,
            self.id
        )
        .fetch_all(pool)
        .await?;
        Ok(res.into())
    }

    #[tracing::instrument]
    async fn ensure_in_db(&self, pool: &PgPool) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
    INSERT INTO rules ( id, name, transaction_filter_qs )
    VALUES ( $1, $2, $3 )
    ON CONFLICT (id) DO UPDATE set name = EXCLUDED.name, transaction_filter_qs = EXCLUDED.transaction_filter_qs
            "#,
            self.id,
            self.name,
            self.transaction_filter_qs,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_by_id(id: &RuleID, pool: &PgPool) -> anyhow::Result<Rule> {
        let res = sqlx::query_as!(
            Rule,
            r#"
        SELECT id, name, transaction_filter_qs
        FROM rules r
        WHERE id = $1
            "#,
            id
        )
        .fetch_one(pool)
        .await?;

        Ok(res)
    }

    pub async fn delete_by_id(id: &RuleID, pool: &PgPool) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
                DELETE FROM rules
                WHERE id = $1
            "#,
            id,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    fn id_string(&self) -> String {
        self.id.as_simple().to_string()
    }

    pub async fn add_label_from_id(
        &self,
        label_id: crate::labels::LabelID,
        pool: &PgPool,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
    INSERT INTO rules_labels ( rule_id, label_id )
    VALUES ( $1, $2 )
    ON CONFLICT (rule_id, label_id) DO NOTHING
            "#,
            self.id,
            label_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn remove_label_by_id(
        &self,
        label_id: crate::labels::LabelID,
        pool: &PgPool,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
    DELETE FROM rules_labels
    WHERE
       rule_id = $1
    AND
       label_id = $2
            "#,
            self.id,
            label_id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}

impl From<Vec<Rule>> for RulesQuery {
    fn from(item: Vec<Rule>) -> Self {
        RulesQuery { item }
    }
}

pub struct RulesQuery {
    item: Vec<Rule>,
}

impl RulesQuery {
    #[tracing::instrument]
    pub async fn all(pool: &PgPool) -> anyhow::Result<Self> {
        let res = sqlx::query_as!(
            Rule,
            r#"
        SELECT id, name, transaction_filter_qs
        FROM rules r
        ORDER BY
            r.id ASC
            "#,
        )
        .fetch_all(pool)
        .await?;
        Ok(res.into())
    }

    pub fn render(self) -> anyhow::Result<maud::Markup> {
        Ok(maud::html! {
            ul {
                @for rule in self.item {
                    li {
                    span {
                        a href={(format!("/rules/{}", rule.id_string()))} { (rule.name) " " (rule.transaction_filter_qs)}

                    }}
                }
            }

        })
    }
}

pub async fn handle_rules(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
) -> Result<Response, crate::AppError> {
    let user_connections_f = Connection::connections(&app_state.db);
    let balances_f = crate::accounts::SFAccountBalanceQueryResult::get_balances(&app_state.db);
    let rules_fut = RulesQuery::all(&app_state.db);

    let (user_connections, balances, rules_result) =
        try_join!(user_connections_f, balances_f, rules_fut)?;

    Ok(html::maud_page(html! {
          div class="flex flex-col lg:flex-row"{
          (html::sidebar(user_connections, balances))
          div #main class="main" {

            div {
                h3 { "Rules"}
                p { "Save a rule query from the main page"}
            }
            (&rules_result.render()?)
          }}

    })
    .into_response())
}

pub async fn handle_rules_fragment(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
) -> Result<Response, crate::AppError> {
    let rules_result = RulesQuery::all(&app_state.db).await?;
    let f = TransactionsFilterOptions::default();
    Ok(html! {
        div {
          h3 { "Rules"}
          p { "Save a rule query from the main page"}
        }
        (&rules_result.render()?)

    }
    .into_response())
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct QSRuleID {
    pub rule_id: RuleID,
}
pub async fn handle_rule(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
    Path(params): Path<QSRuleID>,
) -> Result<Response, crate::AppError> {
    let user_connections_f = Connection::connections(&app_state.db);
    let balances_f = crate::accounts::SFAccountBalanceQueryResult::get_balances(&app_state.db);
    let rule_f = Rule::get_by_id(&params.rule_id, &app_state.db);
    let rule_labels_f = crate::labels::LabelsQuery::for_rule(&params.rule_id, &app_state.db);

    let (user_connections, balances, rule, rule_labels) =
        try_join!(user_connections_f, balances_f, rule_f, rule_labels_f)?;

    let rule_id = rule.id_string();

    Ok(html::maud_page(html! {
          div class="flex flex-col lg:flex-row"{
          (html::sidebar(user_connections, balances))
          div #main class="main" {

            div {
                h3 { "Rule"}
                p { "definition:"}
                p { (rule.transaction_filter_qs)}

                form
                    hx-get={(format!("/f/rules/{rule_id}/labels/search")) }
                    hx-target={"#search-results-rule"}
                    hx-trigger={"input changed delay:100ms throttle:50ms from:input#search-input-rule" }
                {
                    input #{ "search-input-rule" }
                        name="search"
                        placeholder="Begin typing to search labels"
                    {}
                    ul #{"search-results-rule" } {}

                }
                (render_label_list(&rule, rule_labels))

                form
                    hx-delete={"/rules/" (rule_id)}
                    hx-redirect="/rules"
                    hx-confirm="Delete rule?"
                {
                    input type="submit" class="border" { "DELETE Rule"}
                }
            }
        }}
    })
    .into_response())
}
pub async fn handle_rule_delete(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
    Path(params): Path<QSRuleID>,
) -> Result<Response, crate::AppError> {
    Rule::delete_by_id(&params.rule_id, &app_state.db).await?;
    let mut headers = HeaderMap::new();
    headers.insert("HX-Redirect", "/rules".parse().unwrap());
    Ok((
        headers,
        // Redirect::to("/rules"),  // check if not htmx and use this instead
    )
        .into_response())
}

fn render_label_list(rule: &Rule, rule_labels: crate::labels::LabelsQuery) -> maud::Markup {
    let rule_id = rule.id_string();

    maud::html! {
                div #"labels-list" {

                    h4 { "Current labels" }
                    @for label in rule_labels.item {
                        @let del_url = format!("/f/rules/{rule_id}/labels/{}", label.id);
                        form
                            hx-target="#labels-list"
                            hx-swap="outerHTML"
                            hx-delete={(del_url)}
                            hx-trigger="click"
                        {
                            (crate::svg_icon::x_circle())
                             (label.label)
                        }
                    }
                }
    }
}

pub fn new_from_filter_options_html_box(
    txf: TransactionsFilterOptions,
) -> anyhow::Result<maud::Markup> {
    let _qs = txf.clone();
    //TODO: use txf to construct input fields
    Ok(maud::html! {
    form
          method="post"
          action="/f/rules/new"
     {
      input type="submit" class="border" { "New Rule"}
      (txf.render_to_hidden_input_fields())

    }})
}

pub async fn handle_new_rule_fragment(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
    tx_filter: Form<TransactionsFilterOptions>,
) -> Result<Response, crate::AppError> {
    let filter_options = tx_filter.deref();
    let name = "New Rule".to_string();
    let rule = Rule::try_new(name, filter_options.to_owned())?;
    rule.ensure_in_db(&app_state.db).await?;

    let rule_id = rule.id_string();

    Ok(Redirect::to(&format!("/rules/{rule_id}")).into_response())
}

pub async fn handle_labels_search_fragment(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
    Path(params): Path<QSRuleID>,
    Form(form): Form<crate::labels::LabelSearch>,
) -> Result<Response, crate::AppError> {
    let rule_id = params.rule_id;
    let results = crate::labels::LabelsQuery::search(form.search, &app_state.db).await?;
    Ok(html! {(results.render_add_labels_for_rule(rule_id))}.into_response())
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct LabelIDForm {
    label_id: crate::labels::LabelID,
}

pub async fn handle_label_add(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
    Path(params): Path<QSRuleID>,
    Form(form): Form<LabelIDForm>,
) -> Result<Response, crate::AppError> {
    let rule_id = params.rule_id;
    let rule = Rule::get_by_id(&rule_id, &app_state.db).await?;
    rule.add_label_from_id(form.label_id, &app_state.db).await?;

    let rule_labels = crate::labels::LabelsQuery::for_rule(&params.rule_id, &app_state.db).await?;

    //TODO REturn updated search... I guess? not the current labels
    Ok(html! {(render_label_list(&rule, rule_labels))}.into_response())
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct RuleLabelID {
    pub rule_id: RuleID,
    pub label_id: crate::labels::LabelID,
}
pub async fn handle_label_delete(
    State(app_state): State<AppState>,
    _user: service_conventions::oidc::OIDCUser,
    Path(params): Path<RuleLabelID>,
) -> Result<Response, crate::AppError> {
    let rule_id = params.rule_id;
    let label_id = params.label_id;
    let rule = Rule::get_by_id(&rule_id, &app_state.db).await?;
    rule.remove_label_by_id(label_id, &app_state.db).await?;

    let rule_labels = crate::labels::LabelsQuery::for_rule(&params.rule_id, &app_state.db).await?;

    //TODO REturn updated search... I guess? not the current labels
    Ok(html! {(render_label_list(&rule, rule_labels))}.into_response())
}

pub async fn try_apply_rules(app_state: &crate::AppState) -> anyhow::Result<()> {
    let rules = RulesQuery::all(&app_state.db_spike).await?;

    for rule in rules.item {
        let filter = rule.get_transaction_filter()?;
        let labels = rule.get_labels(&app_state.db_spike).await?;
        let txs = tx::SFAccountTXQuery::from_filter_options(&filter, &app_state.db_spike).await?;
        for transaction in txs.item {
            for label in labels.clone() {
                tracing::debug!(tx= ?transaction, label= ?label, "Adding label");
                tx::AccountTransactionLabel::ensure_tx_has_label(
                    transaction.id,
                    &label.id,
                    &app_state.db_spike,
                )
                .await?;
            }
        }
    }

    Ok(())
}
