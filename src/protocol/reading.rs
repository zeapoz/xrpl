use std::{io, net::SocketAddr};

use pea2pea::{protocols::Reading, ConnectionSide, Pea2Pea};
use tracing::*;

use crate::{
    protocol::codecs::binary::{BinaryCodec, BinaryMessage},
    tools::synthetic_node::SyntheticNode,
};

#[async_trait::async_trait]
impl Reading for SyntheticNode {
    type Message = BinaryMessage;
    type Codec = BinaryCodec;

    fn codec(&self, _addr: SocketAddr, _side: ConnectionSide) -> Self::Codec {
        Self::Codec::new(self.node().span().clone())
    }

    async fn process_message(&self, source: SocketAddr, message: Self::Message) -> io::Result<()> {
        info!(parent: self.node().span(), "read a message from {}: {:?}", source, message.payload);

        Ok(())
    }
}
