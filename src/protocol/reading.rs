use std::{io, net::SocketAddr};

use pea2pea::{protocols::Reading, ConnectionSide, Pea2Pea};
use tracing::*;

use crate::{
    protocol::codecs::message::{BinaryMessage, MessageCodec},
    tools::inner_node::InnerNode,
};

#[async_trait::async_trait]
impl Reading for InnerNode {
    type Message = BinaryMessage;
    type Codec = MessageCodec;

    fn codec(&self, _addr: SocketAddr, _side: ConnectionSide) -> Self::Codec {
        Self::Codec::new(self.node().span().clone())
    }

    async fn process_message(&self, source: SocketAddr, message: Self::Message) -> io::Result<()> {
        debug!(parent: self.node().span(), "read a message from {}: {:?}", source, message.payload);
        debug!(
            parent: self.node().span(),
            "sending the message to the node's inbound queue"
        );
        self.sender
            .send((source, message))
            .await
            .expect("receiver dropped");
        Ok(())
    }
}
