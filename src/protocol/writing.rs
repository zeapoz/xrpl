use std::{io, net::SocketAddr};

use bytes::{BufMut, BytesMut};
use pea2pea::{protocols::Writing, ConnectionSide, Pea2Pea};
use tokio_util::codec::Encoder;

use crate::{
    protocol::codecs::binary::{BinaryCodec, Payload},
    tools::inner_node::InnerNode,
};

impl Encoder<Vec<u8>> for BinaryCodec {
    type Error = io::Error;

    fn encode(&mut self, message: Vec<u8>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.put_slice(&message);

        Ok(())
    }
}

impl Encoder<MessageOrBytes> for BinaryCodec {
    type Error = io::Error;

    fn encode(&mut self, message: MessageOrBytes, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match message {
            MessageOrBytes::Payload(msg) => Encoder::<Payload>::encode(self, msg, dst),
            MessageOrBytes::Bytes(msg) => Encoder::<Vec<u8>>::encode(self, msg, dst),
        }
    }
}

pub enum MessageOrBytes {
    Payload(Payload),
    Bytes(Vec<u8>),
}

impl Writing for InnerNode {
    type Message = MessageOrBytes;
    type Codec = BinaryCodec;

    fn codec(&self, _addr: SocketAddr, _side: ConnectionSide) -> Self::Codec {
        Self::Codec::new(self.node().span().clone())
    }
}
