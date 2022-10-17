use std::time::Duration;

use reqwest::{
    header::{ACCEPT, CONTENT_TYPE},
    Client, RequestBuilder,
};
use serde::{Deserialize, Serialize};
use tokio::time::{error::Elapsed, sleep, timeout};

use crate::tools::constants::EXPECTED_RESULT_TIMEOUT;

const API_VERSION: u32 = 1;

pub async fn wait_for_state(rpc_url: &str, state: String) {
    tokio::time::timeout(EXPECTED_RESULT_TIMEOUT, async move {
        loop {
            if let Ok(response) = get_server_info(rpc_url).await {
                if response.result.info.server_state == state {
                    break;
                }
            }
            sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .unwrap()
}

pub async fn wait_for_account_data(
    rpc_url: &str,
    account: &str,
    timeout: Duration,
) -> Result<RpcResponse<AccountInfoResponse>, Elapsed> {
    tokio::time::timeout(timeout, async move {
        loop {
            if let Ok(account_data) = get_account_info(rpc_url, account).await {
                return account_data;
            }
            sleep(Duration::from_millis(100)).await;
        }
    })
    .await
}

pub async fn wait_for_ledger_info(
    rpc_url: &str,
) -> Result<RpcResponse<LedgerInfoResponse>, Elapsed> {
    timeout(EXPECTED_RESULT_TIMEOUT, async {
        loop {
            if let Ok(info) = get_ledger_info(rpc_url).await {
                return info;
            }
            sleep(Duration::from_millis(250)).await;
        }
    })
    .await
}

async fn execute_rpc<T: for<'a> Deserialize<'a>>(
    rpc_url: &str,
    body: &impl Serialize,
) -> anyhow::Result<T> {
    let response = build_json_request(rpc_url, body).send().await?;
    Ok(response.error_for_status()?.json::<T>().await?)
}

async fn get_account_info(
    rpc_url: &str,
    account: &str,
) -> anyhow::Result<RpcResponse<AccountInfoResponse>> {
    execute_rpc(rpc_url, &build_account_info_request(account)).await
}

async fn get_server_info(rpc_url: &str) -> anyhow::Result<RpcResponse<ResultResponse>> {
    let request: RpcRequest<Option<()>> = RpcRequest {
        id: String::from("1"),
        method: String::from("server_info"),
        api_version: API_VERSION,
        params: None,
    };
    execute_rpc(rpc_url, &request).await
}

pub async fn get_transaction_info(
    rpc_url: &str,
    transaction: String,
) -> anyhow::Result<RpcResponse<TransactionInfoResponse>> {
    execute_rpc(rpc_url, &build_transaction_info_request(transaction)).await
}

pub async fn get_ledger_info(rpc_url: &str) -> anyhow::Result<RpcResponse<LedgerInfoResponse>> {
    let request = RpcRequest {
        id: String::from("1"),
        method: String::from("ledger"),
        api_version: API_VERSION,
        params: vec![LedgerInfoRequest {
            ledger_index: "validated".to_string(),
            accounts: false,
            full: false,
            transactions: false,
            expand: false,
            owner_funds: false,
        }],
    };
    execute_rpc(rpc_url, &request).await
}

pub async fn submit_transaction(
    rpc_url: &str,
    tx_blob: String,
    fail_hard: bool,
) -> anyhow::Result<RpcResponse<SubmitTransactionResponse>> {
    let request = build_submit_transaction_request(tx_blob, fail_hard);
    execute_rpc(rpc_url, &request).await
}

#[derive(Serialize)]
struct LedgerInfoRequest {
    ledger_index: String,
    accounts: bool,
    full: bool,
    transactions: bool,
    expand: bool,
    owner_funds: bool,
}

fn build_transaction_info_request(transaction: String) -> RpcRequest<Vec<TransactionInfoRequest>> {
    RpcRequest {
        id: String::from("1"),
        method: String::from("tx"),
        api_version: API_VERSION,
        params: vec![TransactionInfoRequest {
            transaction,
            binary: false,
        }],
    }
}

fn build_submit_transaction_request(
    tx_blob: String,
    fail_hard: bool,
) -> RpcRequest<Vec<SubmitTransactionRequest>> {
    RpcRequest {
        id: String::from("1"),
        method: String::from("submit"),
        api_version: API_VERSION,
        params: vec![SubmitTransactionRequest { tx_blob, fail_hard }],
    }
}

fn build_json_request(rpc_url: &str, request: &impl Serialize) -> RequestBuilder {
    Client::new()
        .post(rpc_url)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .json(request)
}

fn build_account_info_request(account: &str) -> RpcRequest<Vec<AccountInfoRequestParams>> {
    RpcRequest {
        id: String::from("1"),
        method: String::from("account_info"),
        api_version: API_VERSION,
        params: vec![AccountInfoRequestParams {
            account: String::from(account),
            ledger_index: String::new(),
            queue: false,
            signer_lists: false,
            strict: false,
        }],
    }
}

#[derive(Serialize)]
struct SubmitTransactionRequest {
    tx_blob: String,
    fail_hard: bool,
}

#[derive(Debug, Deserialize)]
pub struct SubmitTransactionResponse {
    pub accepted: bool,
    pub applied: bool,
    pub broadcast: bool,
}

#[derive(Serialize)]
struct TransactionInfoRequest {
    transaction: String,
    binary: bool,
}

#[derive(Debug, Deserialize)]
pub struct TransactionInfoResponse {
    // Empty struct as we aren't interested in content
}

#[derive(Serialize)]
struct AccountInfoRequestParams {
    account: String,
    ledger_index: String,
    queue: bool,
    signer_lists: bool,
    strict: bool,
}

#[derive(Serialize)]
struct RpcRequest<T> {
    id: String,
    method: String,
    api_version: u32,
    params: T,
}

#[derive(Debug, Deserialize)]
pub struct RpcResponse<T> {
    pub result: T,
}

#[derive(Debug, Deserialize)]
pub struct ResultResponse {
    pub info: ServerInfoResponse,
}

#[derive(Debug, Deserialize)]
pub struct ServerInfoResponse {
    pub server_state: String,
}

#[derive(Debug, Deserialize)]
pub struct AccountInfoResponse {
    pub account_data: AccountDataResponse,
}

#[derive(Debug, Deserialize)]
pub struct AccountDataResponse {
    #[serde(rename(deserialize = "Balance"))]
    pub balance: String,

    #[allow(dead_code)]
    #[serde(rename(deserialize = "PreviousTxnID"))]
    pub previous_transaction: String,
}

#[derive(Debug, Deserialize)]
pub struct LedgerInfoResponse {
    pub ledger: LedgerResponseData,
}

#[derive(Debug, Deserialize)]
pub struct LedgerResponseData {
    pub ledger_hash: String,
    pub ledger_index: String,
}
