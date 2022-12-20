use std::net::SocketAddr;

use clap::Parser;

#[derive(Debug, Parser)]
#[clap(author, version, about, long_about = None)]
pub(super) struct Args {
    /// The initial addresses to connect to
    #[clap(short, long, value_parser, num_args = 1.., required = true)]
    seed_addrs: Vec<SocketAddr>,
}
