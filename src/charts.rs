use axum::{
    extract::{FromRef, Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Form, Router,
};
use axum_extra::extract::Query;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

use charts_rs::{Box, LineChart, SeriesCategory, THEME_GRAFANA};

#[derive(Deserialize, Debug, Clone, Default, Serialize)]
pub struct ChartOptions {
    #[serde(default)]
    x_size: XSize,
    #[serde(default)]
    y_size: YSize,
}

#[derive(Deserialize, Debug, Clone, Serialize)]
struct XSize(u32);
impl Default for XSize {
    fn default() -> Self {
        Self(720)
    }
}
#[derive(Deserialize, Debug, Clone, Serialize)]
struct YSize(u32);
impl Default for YSize {
    fn default() -> Self {
        Self(240)
    }
}

pub async fn get_chart(
    State(app_state): State<crate::AppState>,
    user: service_conventions::oidc::OIDCUser,
    chart_options: Query<ChartOptions>,
    tx_filter: Query<crate::TransactionsFilterOptions>,
) -> Result<Response, crate::AppError> {
    let d = crate::tx::SFAccountTXQuery::from_options_group_by(
        tx_filter.deref().clone().into(),
        &app_state.db,
    )
    .await?;
    let vals_opt: Option<Vec<f32>> = d.clone().into_iter().map(|i| i.amount_f32()).collect();
    let vals = vals_opt.expect("No data?");
    println!("{vals:?}");
    let dates_opt: Option<Vec<String>> = d.into_iter().map(|i| i.name()).collect();
    let dates = dates_opt.expect("No data?");

    //let dates = d.into_iter().map(|i| i.interval);

    let mut chart = LineChart::new_with_theme(vec![("vals", vals).into()], dates, THEME_GRAFANA);
    chart.width = chart_options.x_size.0 as f32;
    chart.height = chart_options.y_size.0 as f32;
    chart.margin.left = 15.0;
    chart.margin.right = 15.0;
    //bar_chart.title_text = "Mixed Line and Bar".to_string();
    //bar_chart.legend_margin = Some(Box {
    //    top: bar_chart.title_height,
    //    bottom: 5.0,
    //    ..Default::default()
    //});
    //bar_chart.series_list[2].category = Some(SeriesCategory::Line);
    //bar_chart.series_list[2].y_axis_index = 1;
    //bar_chart.series_list[2].label_show = true;

    //bar_chart
    //    .y_axis_configs
    //    .push(bar_chart.y_axis_configs[0].clone());
    //bar_chart.y_axis_configs[0].axis_formatter = Some("USD".to_string());

    let data = &chart.svg()?.to_owned();

    Ok(data.clone().into_response())
}
