use crate::svg_icon;
use maud::{html, DOCTYPE};

pub fn maud_page(content: maud::Markup) -> maud::Markup {
    html! {
       (DOCTYPE)
       (maud_header())
       (maud_body(content))
    }
}

fn maud_header() -> maud::Markup {
    html! {
        link rel="stylesheet" href="/static/tailwind.css";
        script src="/static/htmx-2.0.0.js" {}
        meta name="viewport" content="width=device-width, initial-scale=1.0";
    }
}

fn maud_nav() -> maud::Markup {
    html! {
        nav class="nav bg-gray-100" {

            div class="flex lg:flex-1 m-1" {
                a class="no-underline hover:no-underline font-extrabold m-3 text-2xl" href="/" { "Expense Tracker" }

            };
        }
    }
}

fn maud_body(content: maud::Markup) -> maud::Markup {
    html! {
        body {
            (maud_nav())
            div class="w-full lg:max-w-screen-xl lg:flex-auto mx-auto pt-20 lg:place-content-center" {

                div class="w-full px-2 lg:px-6 leading-normal" {
                        (content)
                };
            };
        };
    }
}

pub fn sidebar(
    user_connections: Vec<crate::Connection>,
    balances: Vec<crate::accounts::SFAccountBalanceQueryResult>,
) -> maud::Markup {
    html! {
        div class="sidebar" {
          h2 { "Connections:" }
          @for sfconn in &user_connections {
          div

              hx-get={"/f/connection/" (sfconn.id) }
              hx-target="this"
              hx-swap="innerHTML"
              hx-trigger="load"
              {
                (sfconn.id)
              }
          }
          div {
            h3 { "Add a SimpleFin Connection"}
            form method="post" action="/simplefin-connection/add" {
              input id="simplefin_token" class="border min-w-full" name="simplefin_token" {}
              input type="submit" class="border" {}
          }
          }


          p
                  hx-get="/f/transactions"
                  hx-push-url={"/"}
                  hx-target="#main"
                  hx-swap="innerHTML"
                  hx-trigger="click"
          { "Transactions Page"}
          p
                  hx-get="/f/labels"
                  hx-push-url={"/labels"}
                  hx-target="#main"
                  hx-swap="innerHTML"
                  hx-trigger="click"
          { "Labels Page"}

          h2 { "Accounts:" }

          table class="table-auto"{
              thead {
                tr
                {
                    th {}
                    th { "Account"}
                    th { "Balance"}
                }
              }
              tbody {
                  (render_balances(balances))
                  tr {
                      td{
                        p
                        hx-get={"/balances/f?active=false"}
                        hx-swap="outerHTML"
                        hx-target="closest tr"
                        hx-trigger="click"
                        {"Show inactive accounts"}}

                }

              }

          }
        }
    }
}

pub fn render_balances(
    balances: Vec<crate::accounts::SFAccountBalanceQueryResult>,
) -> maud::Markup {
    maud::html!(
              @for balance in &balances {
              tr
                  {
                  td
                    hx-get={"/f/transactions?account_id=" (balance.account_id) }
                    hx-push-url={"/?account_id=" (balance.account_id) }
                    hx-target="#main"
                    hx-swap="innerHTML"
                    hx-trigger="click"
                    class="peer"
                    {
                        @if let Some(name) = balance.custom_name.clone() {
                            (name)
                        } @else {
                            (balance.name)
                        }
                    }

                  td
                    class="peer"
                    { (balance.balance.to_decimal(2))}
                  td
                    hx-get={"/f/accounts/" (balance.account_id) }
                    hx-push-url={"/accounts/" (balance.account_id) }
                    hx-target="#main"
                    hx-swap="innerHTML"
                    hx-trigger="click"
                    class="invisible peer-hover:visible hover:visible"
                    { (svg_icon::pencil_square())}
              }
              }
    )
}
