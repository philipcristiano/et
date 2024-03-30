use base64::prelude::*;
use chrono::serde::{ts_seconds, ts_seconds_option};
use reqwest;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SimpleFinAPIError {
    #[error("Error base64 decoding token")]
    TokenDecodeError(#[from] base64::DecodeError),
    #[error(transparent)]
    RequestError(#[from] reqwest::Error),
    #[error(transparent)]
    JSONParseError(#[from] serde_json::Error),
}
#[derive(Deserialize, Serialize, Debug)]
pub struct AccountSet {
    pub errors: Vec<String>,
    pub accounts: Vec<Account>,
}
pub type AccountID = String;
pub type AccountName = String;
pub type Currency = String;
#[derive(Deserialize, Serialize, Debug)]
pub struct Account {
    pub org: Organization,
    pub id: AccountID,
    pub name: AccountName,
    pub currency: Currency,
    #[serde(with = "rust_decimal::serde::float")]
    pub balance: rust_decimal::Decimal, // TODO make a numeric type
    #[serde(
        alias = "available-balance",
        with = "rust_decimal::serde::float_option"
    )]
    pub available_balance: Option<rust_decimal::Decimal>,
    #[serde(with = "ts_seconds", alias = "balance-date")]
    pub balance_date: chrono::DateTime<chrono::Utc>,
    pub transactions: Vec<Transaction>,
}

pub type OrganizationName = String;
#[derive(Deserialize, Serialize, Debug)]
pub struct Organization {
    pub domain: Option<String>,
    #[serde(alias = "sfin-url")]
    pub sfin_url: String,
    pub name: Option<OrganizationName>,
}

pub type TransactionID = String;
#[derive(Deserialize, Serialize, Debug)]
pub struct Transaction {
    pub id: TransactionID,
    #[serde(with = "ts_seconds")]
    pub posted: chrono::DateTime<chrono::Utc>,
    #[serde(with = "rust_decimal::serde::float")]
    pub amount: rust_decimal::Decimal,
    pub description: String,
    #[serde(with = "ts_seconds_option")]
    pub transacted_at: Option<chrono::DateTime<chrono::Utc>>,
    pub pending: Option<bool>,
}

pub async fn token_to_access_url(b64token: String) -> Result<String, SimpleFinAPIError> {
    let claim_url_bytes = BASE64_STANDARD.decode(b64token)?;
    let claim_url = String::from_utf8_lossy(&claim_url_bytes).into_owned();
    let client = reqwest::Client::new();
    let access_url = client.post(claim_url).send().await?.text().await?;
    Ok(access_url)
}

pub async fn accounts(access_url: &String) -> Result<AccountSet, SimpleFinAPIError> {
    let client = reqwest::Client::new();
    let url = format!("{access_url}/accounts");
    let query = vec![("start-date", "0")];
    let account_set_text = client.get(url).query(&query).send().await?.text().await?;

    let account_set: AccountSet = serde_json::from_str(&account_set_text)?;
    Ok(account_set)
}
