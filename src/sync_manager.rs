use crate::{
    accounts::{SFAccount, SFAccountBalance},
    tx::SFAccountTransaction,
    AppState, Connection,
};
use std::time::Duration;
use tokio::time::sleep;
use tracing::Level;

pub async fn sync_all(app_state: AppState) -> () {
    loop {
        let res = try_sync_all(&app_state).await;
        if let Err(e) = res {
            tracing::event!(Level::ERROR, error = e.to_string(), "Could not sync");
        }
        if app_state.features.apply_rules {
            tracing::event!(Level::INFO, "Starting rule application");
            let res = crate::rules::try_apply_rules(&app_state).await;
        }
        sleep(Duration::from_secs(60 * 60)).await;
    }
}

struct Lock {
    pg_try_advisory_xact_lock: Option<bool>,
}

impl Lock {
    fn held(&self) -> bool {
        if let Some(b) = self.pg_try_advisory_xact_lock {
            return b;
        }
        return false;
    }
}

#[tracing::instrument(name = "sync_connections", skip_all)]
async fn try_sync_all(app_state: &AppState) -> anyhow::Result<()> {
    let k = sqlx::postgres::PgAdvisoryLock::new("Sync connections")
        .key()
        .as_bigint();
    let mut c = app_state.db_spike.begin().await?;
    let lock = sqlx::query_as!(Lock, "SELECT pg_try_advisory_xact_lock($1)", k)
        .fetch_one(c.as_mut())
        .await?;
    if lock.held() {
        tracing::event!(Level::DEBUG, "Holding PG Advisory xact lock");
        sync_all_connections(app_state).await?
    } else {
        tracing::event!(Level::INFO, "Could not get PG Advisory lock");
    }
    c.rollback().await?;
    Ok(())
}

async fn sync_all_connections(app_state: &AppState) -> anyhow::Result<()> {
    for conn in Connection::connections(&app_state.db_spike).await? {
        tracing::event!(
            Level::INFO,
            connection_id = conn.id.to_string(),
            "Syncing connection"
        );
        let sync_result = sync_connection(&app_state, &conn).await;
        if let Err(e) = sync_result {
            tracing::error!(error= ?e, "Sync error")
        }
    }
    Ok(())
}

#[tracing::instrument(skip_all, fields(connection_id=sfc.id.to_string()))]
async fn sync_connection(app_state: &AppState, sfc: &Connection) -> anyhow::Result<()> {
    if let Some(sync_info) = sfc.last_sync_info(&app_state.db_spike).await? {
        if sync_info.is_since(chrono::Duration::hours(4)) {
            tracing::event!(
                Level::INFO,
                connection_id = sfc.id.to_string(),
                "Connection synchronized recently, not attempting to sync"
            );
            return Ok(());
        }
    }

    let account_set_result = crate::simplefin_api::accounts(&sfc.access_url).await;
    match account_set_result {
        Err(e) => {
            sfc.mark_synced(&vec![e.to_string()], &app_state.db_spike)
                .await?;
            Err(e)?;
            Ok(())
        }
        Ok(account_set) => {
            for account in account_set.accounts {
                tracing::debug!(
                    account_id = account.id,
                    name = account.name,
                    "Proccessing accout"
                );
                let sfa = SFAccount {
                    simplefin_id: account.id,
                    connection_id: sfc.id,
                    name: account.name,
                    currency: account.currency,
                };
                let et_account = sfa.ensure_in_db(&app_state.db).await?;
                if et_account.active {
                    let sfab = SFAccountBalance {
                        account_id: et_account.id,
                        timestamp: account.balance_date,
                        balance: account.balance,
                    };
                    sfab.ensure_in_db(&app_state.db).await?;

                    let txs_f = account.transactions.iter().map(|src_tx| {
                        let tx = SFAccountTransaction::from_transaction(&et_account, &src_tx);
                        tracing::debug!(simplefine_tx= ?src_tx, et_tx= ?tx, "Account transaction");
                        SFAccountTransaction::ensure_in_db(tx, &app_state.db_spike)
                    });

                    futures::future::try_join_all(txs_f).await?;
                    ()
                } else {
                    tracing::debug!(
                        account_id = et_account.id.clone().to_string(),
                        "Account inactive, not saving transactions"
                    );
                }
            }
            sfc.mark_synced(&account_set.errors, &app_state.db_spike)
                .await?;
            Ok(())
        }
    }
}
