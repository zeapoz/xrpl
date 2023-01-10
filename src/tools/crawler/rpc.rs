use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};

use jsonrpsee::{
    server::{ServerBuilder, ServerHandle},
    RpcModule,
};
use tracing::debug;

use crate::metrics::NetworkSummary;

pub struct RpcContext(Arc<Mutex<NetworkSummary>>);

impl RpcContext {
    /// Creates a new RpcContext.
    pub(crate) fn new(network_summary: Arc<Mutex<NetworkSummary>>) -> RpcContext {
        RpcContext(network_summary)
    }
}

pub async fn initialize_rpc_server(rpc_addr: SocketAddr, rpc_context: RpcContext) -> ServerHandle {
    let server = ServerBuilder::default().build(rpc_addr).await.unwrap();
    let module = create_rpc_module(rpc_context);

    debug!("Starting RPC server at {:?}", server.local_addr().unwrap());
    let server_handle = server.start(module).unwrap();

    debug!("RPC server was successfully started");
    server_handle
}

fn create_rpc_module(rpc_context: RpcContext) -> RpcModule<RpcContext> {
    let mut module = RpcModule::new(rpc_context);
    module
        .register_method("getmetrics", |_, rpc_context| {
            Ok(rpc_context.0.lock().unwrap().clone())
        })
        .unwrap();
    module
}
