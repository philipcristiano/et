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
        script src="/static/htmx-1.9.11.js" {}
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
          div {
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
                    th { "Account"}
                    th { "Balance"}
                }
              }
              tbody {
              @for balance in &balances {
              tr
                  hx-get={"/f/transactions?account_id=" (balance.account_id) }
                  hx-push-url={"/?account_id=" (balance.account_id) }
                  hx-target="#main"
                  hx-swap="innerHTML"
                  hx-trigger="click"
                  {
                  td { (balance.name)}
                  td { (balance.balance.to_decimal(2))}
              }
              }
              }

          }
        }
    }
}
