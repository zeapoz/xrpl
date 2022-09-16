use std::time::Duration;

use reqwest::{
    header::{ACCEPT, CONTENT_TYPE},
    Client, RequestBuilder,
};
use serde::{Deserialize, Serialize};
use tokio::time::error::Elapsed;

use crate::tools::constants::{EXPECTED_RESULT_TIMEOUT, JSON_RPC_ADDRESS};

const API_VERSION: u32 = 1;

pub async fn wait_for_state(state: String) {
    tokio::time::timeout(EXPECTED_RESULT_TIMEOUT, async move {
        loop {
            if let Ok(response) = get_server_info().await {
                if response.result.info.server_state == state {
                    break;
                }
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
    .unwrap()
}

pub async fn wait_for_account_data(
    account: &str,
) -> Result<RpcResponse<AccountInfoResponse>, Elapsed> {
    tokio::time::timeout(EXPECTED_RESULT_TIMEOUT, async move {
        loop {
            if let Ok(account_data) = get_account_info(account).await {
                return account_data;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    })
    .await
}

async fn get_account_info(account: &str) -> anyhow::Result<RpcResponse<AccountInfoResponse>> {
    let response = build_json_request(&build_account_info_request(account))
        .send()
        .await?;
    Ok(response
        .error_for_status()?
        .json::<RpcResponse<AccountInfoResponse>>()
        .await?)
}

async fn get_server_info() -> anyhow::Result<RpcResponse<ResultResponse>> {
    let response = build_json_request(&build_server_info_request())
        .send()
        .await?;
    Ok(response
        .error_for_status()?
        .json::<RpcResponse<ResultResponse>>()
        .await?)
}

// TODO make get_*_info generic
pub async fn get_transaction_info(
    transaction: String,
) -> anyhow::Result<RpcResponse<TransactionInfoResponse>> {
    let response = build_json_request(&build_transaction_info_request(transaction))
        .send()
        .await?;
    Ok(response
        .error_for_status()?
        .json::<RpcResponse<TransactionInfoResponse>>()
        .await?)
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

fn build_json_request(request: &impl Serialize) -> RequestBuilder {
    Client::new()
        .post(JSON_RPC_ADDRESS)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .json(request)
}

fn build_server_info_request() -> RpcRequest<Option<()>> {
    RpcRequest {
        id: String::from("1"),
        method: String::from("server_info"),
        api_version: API_VERSION,
        params: None,
    }
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
