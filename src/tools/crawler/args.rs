use std::net::SocketAddr;

use clap::Parser;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
pub(super) struct Args {
    /// The initial addresses to connect to
    #[clap(short, long, value_parser, num_args = 1.., required = true)]
    pub(super) seed_addrs: Vec<SocketAddr>,

    /// If present, start an RPC server at the specified address
    #[clap(short, long, value_parser)]
    pub(super) rpc_addr: Option<SocketAddr>,
}
