use reqwest::{
    header::{ACCEPT, CONTENT_TYPE},
    Client, RequestBuilder,
};
use serde::{Deserialize, Serialize};

use crate::tools::constants::{EXPECTED_RESULT_TIMEOUT, JSON_RPC_ADDRESS};

const API_VERSION: u32 = 1;

pub async fn wait_for_state(state: String) {
    tokio::time::timeout(EXPECTED_RESULT_TIMEOUT, async move {
        loop {
            if let Ok(response) = get_server_state().await {
                if response.result.info.server_state == state {
                    break;
                }
            }
        }
    })
    .await
    .unwrap()
}

async fn get_server_state() -> anyhow::Result<RpcResponse> {
    let response = build_json_request(&build_server_info_request())
        .send()
        .await?;
    Ok(response.error_for_status()?.json::<RpcResponse>().await?)
}

fn build_json_request(request: &impl Serialize) -> RequestBuilder {
    Client::new()
        .post(JSON_RPC_ADDRESS)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCEPT, "application/json")
        .json(request)
}

fn build_server_info_request() -> RpcRequest {
    RpcRequest {
        id: String::from("1"),
        method: String::from("server_info"),
        api_version: API_VERSION,
    }
}

#[derive(Serialize)]
struct RpcRequest {
    id: String,
    method: String,
    api_version: u32,
}

#[derive(Deserialize)]
pub struct RpcResponse {
    pub result: ResultResponse,
}

#[derive(Deserialize)]
pub struct ResultResponse {
    pub info: InfoResponse,
}

#[derive(Deserialize)]
pub struct InfoResponse {
    pub server_state: String,
}
