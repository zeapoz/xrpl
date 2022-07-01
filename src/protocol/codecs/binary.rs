use std::io;

use bytes::{Buf, BytesMut};
use tokio_util::codec::{Decoder, Encoder};
use tracing::*;

use crate::protocol::proto::*;

const HEADER_LEN_COMPRESSED: u32 = 10;

const HEADER_LEN_UNCOMPRESSED: u32 = 6;

const COMPRESSION_ALGO: u8 = 0xf0;

const COMPRESSION_LZ4: u8 = 0x90;

const COMPRESSED_TRUE: u8 = 0x80;

const COMPRESSED_FALSE: u8 = 0xfc;

const PROTOCOL_ERROR: u8 = 0x0c;

#[derive(Debug)]
enum Compression {
    None,
    LZ4,
}

#[derive(Debug)]
pub struct Header {
    #[allow(dead_code)]
    total_wire_size: u32,
    #[allow(dead_code)]
    header_size: u32,
    payload_wire_size: u32,
    #[allow(dead_code)]
    uncompressed_size: u32,
    message_type: u16,
    #[allow(dead_code)]
    compression: Compression,
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Payload {
    TmManifests(TmManifests),
    TmValidation(TmValidation),
    TmValidatorListCollection(TmValidatorListCollection),
    TmGetPeerShardInfoV2(TmGetPeerShardInfoV2),
}

#[derive(Debug)]
pub struct BinaryMessage {
    pub header: Header,
    pub payload: Payload,
}

pub struct BinaryCodec {
    current_msg_header: Option<Header>,
    // The associated node's span.
    span: Span,
}

impl BinaryCodec {
    pub fn new(span: Span) -> Self {
        Self {
            current_msg_header: None,
            span,
        }
    }
}

impl Decoder for BinaryCodec {
    type Item = BinaryMessage;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            return Ok(None);
        }

        if self.current_msg_header.is_none() {
            if src[0] & COMPRESSED_TRUE != 0 {
                trace!(parent: &self.span, "processing a compressed message");

                let header_size = HEADER_LEN_COMPRESSED;
                if src.remaining() < header_size as usize {
                    return Ok(None);
                }

                // protocol error
                if (src[0] & PROTOCOL_ERROR) != 0 {
                    unimplemented!();
                }

                let header_bytes = src.split_to(header_size as usize);
                let mut iter = header_bytes.into_iter();

                let compression = src[0] & COMPRESSION_ALGO;
                trace!(parent: &self.span, "compression: {:x}", compression);

                // only LZ4 is currently supported
                if compression != COMPRESSION_LZ4 {
                    unimplemented!();
                }

                let mut payload_wire_size = 0;
                for _ in 0..4 {
                    payload_wire_size = (payload_wire_size << 8u32) + iter.next().unwrap() as u32;
                }
                payload_wire_size &= 0x0FFFFFFF; // clear the top four bits (the compression bits)

                let total_wire_size = header_size + payload_wire_size;

                let mut message_type = 0;
                for _ in 0..2 {
                    message_type = (message_type << 8u16) + iter.next().unwrap() as u16;
                }

                let mut uncompressed_size = 0;
                for _ in 0..4 {
                    uncompressed_size = (uncompressed_size << 8u32) + iter.next().unwrap() as u32;
                }

                let header = Header {
                    total_wire_size,
                    header_size,
                    payload_wire_size,
                    uncompressed_size,
                    message_type,
                    compression: Compression::LZ4,
                };

                self.current_msg_header = Some(header);
            } else if src[0] & COMPRESSED_FALSE == 0 {
                trace!(parent: &self.span, "processing an uncompressed message");

                let header_size = HEADER_LEN_UNCOMPRESSED;
                if src.remaining() < header_size as usize {
                    return Ok(None);
                }

                let header_bytes = src.split_to(header_size as usize);
                let mut iter = header_bytes.into_iter();

                let mut payload_wire_size = 0;
                for _ in 0..4 {
                    payload_wire_size = (payload_wire_size << 8u32) + iter.next().unwrap() as u32;
                }

                let uncompressed_size = payload_wire_size;
                let total_wire_size = header_size + payload_wire_size as u32;

                let mut message_type = 0;
                for _ in 0..2 {
                    message_type = (message_type << 8u16) + iter.next().unwrap() as u16;
                }

                let header = Header {
                    total_wire_size,
                    header_size,
                    payload_wire_size,
                    uncompressed_size,
                    message_type,
                    compression: Compression::None,
                };

                self.current_msg_header = Some(header);
            } else {
                error!(parent: &self.span, "invalid compression indicator");

                return Err(io::ErrorKind::InvalidData.into());
            }
        }

        if let Some(Header {
            payload_wire_size, ..
        }) = self.current_msg_header
        {
            if src.remaining() < payload_wire_size as usize {
                return Ok(None);
            }

            let header = self.current_msg_header.take().unwrap();
            let mut payload = src.split_to(payload_wire_size as usize);

            let payload = match header.message_type {
                2 => Payload::TmManifests(prost::Message::decode(&mut payload)?),
                41 => Payload::TmValidation(prost::Message::decode(&mut payload)?),
                56 => Payload::TmValidatorListCollection(prost::Message::decode(&mut payload)?),
                61 => Payload::TmGetPeerShardInfoV2(prost::Message::decode(&mut payload)?),
                _ => unimplemented!(),
            };

            let message = BinaryMessage { header, payload };

            debug!(parent: &self.span, "decoded a header: {:?}", message.header);

            Ok(Some(message))
        } else {
            unreachable!();
        }
    }
}

impl Encoder<BinaryMessage> for BinaryCodec {
    type Error = io::Error;

    fn encode(&mut self, _message: BinaryMessage, _dst: &mut BytesMut) -> Result<(), Self::Error> {
        todo!();
    }
}
