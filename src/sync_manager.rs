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
        sleep(Duration::from_secs(60 * 60)).await;
    }
}

#[tracing::instrument(name = "sync_connections", skip_all)]
async fn try_sync_all(app_state: &AppState) -> anyhow::Result<()> {
    for conn in Connection::connections(&app_state.db_spike).await? {
        tracing::event!(
            Level::INFO,
            connection_id = conn.id.to_string(),
            "Syncing connection"
        );
        sync_connection(&app_state, &conn).await?;
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

    let account_set = crate::simplefin_api::accounts(&sfc.access_url).await?;
    for account in account_set.accounts {
        tracing::debug!(account_id = account.id, "Saving account");
        let sfa = SFAccount {
            simplefin_id: account.id,
            connection_id: sfc.id,
            name: account.name,
            currency: account.currency,
        };
        let et_account = sfa.ensure_in_db(&app_state.db).await?;
        let sfab = SFAccountBalance {
            account_id: et_account.id,
            timestamp: account.balance_date,
            balance: account.balance,
        };
        sfab.ensure_in_db(&app_state.db).await?;

        let txs_f = account.transactions.iter().map(|src_tx| {
            let tx = SFAccountTransaction::from_transaction(&et_account, &src_tx);
            SFAccountTransaction::ensure_in_db(tx, &app_state.db_spike)
        });

        futures::future::try_join_all(txs_f).await?;
        ()
    }
    sfc.mark_synced(&app_state.db_spike).await?;
    Ok(())
}
