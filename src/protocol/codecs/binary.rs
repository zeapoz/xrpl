use std::io;

use bytes::{Buf, BufMut, BytesMut};
use prost::Message;
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

fn pack(dst: &mut [u8], size: u32) {
    dst[0] = ((size >> 24) & 0x0f) as u8;
    dst[1] = ((size >> 16) & 0xff) as u8;
    dst[2] = ((size >> 8) & 0xff) as u8;
    dst[3] = (size & 0xff) as u8;
}

impl Encoder<Payload> for BinaryCodec {
    type Error = io::Error;

    fn encode(&mut self, message: Payload, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let (payload_len, msg_type) = match &message {
            Payload::TmManifests(msg) => {
                (msg.encoded_len() as u32, MessageType::MtManifests as i32)
            }
            Payload::TmValidation(msg) => {
                (msg.encoded_len() as u32, MessageType::MtValidation as i32)
            }
            Payload::TmValidatorListCollection(msg) => (
                msg.encoded_len() as u32,
                MessageType::MtValidatorlistcollection as i32,
            ),
            Payload::TmGetPeerShardInfoV2(msg) => (
                msg.encoded_len() as u32,
                MessageType::MtGetPeerShardInfoV2 as i32,
            ),
        };

        let header_size = HEADER_LEN_UNCOMPRESSED;
        let _header = Header {
            total_wire_size: header_size + payload_len,
            header_size,
            payload_wire_size: payload_len,
            uncompressed_size: payload_len,
            message_type: msg_type as u16,
            compression: Compression::None, // TODO: are compressed messages used?
        };

        let mut header_bytes = [0u8; HEADER_LEN_UNCOMPRESSED as usize];

        pack(&mut header_bytes, payload_len);

        header_bytes[4] = ((msg_type >> 8) & 0xff) as u8;
        header_bytes[5] = (msg_type & 0xff) as u8;

        let mut bytes = BytesMut::new();

        bytes.put(&header_bytes[..]);

        match message {
            Payload::TmManifests(msg) => (msg.encode(&mut bytes).unwrap(),),
            Payload::TmValidation(msg) => (msg.encode(&mut bytes).unwrap(),),
            Payload::TmValidatorListCollection(msg) => (msg.encode(&mut bytes).unwrap(),),
            Payload::TmGetPeerShardInfoV2(msg) => (msg.encode(&mut bytes).unwrap(),),
        };

        dst.put(&*bytes);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_and_encode() {
        // a sample raw message
        let raw = BytesMut::from(
            &b"\0\0\0\xeb\0)\n\xe8\x01\"\x80\0\0\x01&\x01\xbb\xef\xd0)*Qj\":r\
            \x8b\x90\xe3fz`[Q\x1c5@P\xcd\\\x1e0\x0c5\xdd\xe9A\xe2\xf62\xa4\xbev\x81jD)Z~\xda6\xa8\x96\
            \x81H\xa6P\x17\x81\xcc\xe2\x81\x12\xdeF\xba\x9fCj \xc7\xce\x9b\xcb\xf7\x8f\x90$\x1f\x9b\xfb\
            \xc3W\xcd\x1c\\\xb5\xe2x\x12P\x19\x14\xcb1\x9bh\x9cY\\\x02RDF(\x17\xcc\xa8\xf3>\x05\x83\xd8\
            \x14y\xfa\xb6\xdb\xa4\xe0\x1e\x96M4s!\x03f\x98Z*X\xfc\xddd\0J\n\x1b\x0f\xe5\xc7U\x08\x91\
            Cgu\xadPV-\xa6\xdf\xac\xe1:\xe6/vF0D\x02 5(s\x01\x17\x94\x07\x9a\xe9\x9c\x1c\xe9g\x02Y\
            \x9fZ!\x1c\xecg+\\\x11NS\xb8g\x05\x8c;b\x02 {\xe0\x14\xe6\xc7\x91M\xd0#\xf6;'i\xc5\
            \xad\"~\xb2\xdd\xb93\xf7V\xa1Zc\xe2D\xf8\x8bf\xd3"[..],
        );

        let mut codec = BinaryCodec::new(Span::none());
        let msg = codec.decode(&mut raw.clone()).unwrap().unwrap();

        let mut encoded = BytesMut::new();
        codec.encode(msg.payload, &mut encoded).unwrap();

        assert_eq!(raw, encoded);
    }
}
