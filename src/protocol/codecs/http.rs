/// This codec is used during the handshake.
use std::io;

use bytes::{Buf, Bytes, BytesMut};
use tokio_util::codec::{BytesCodec, Decoder, Encoder};
use tracing::*;

// The HTTP message type;
pub enum HttpMsg {
    Request,
    Response,
}

// A codec used to handle HTTP messages.
pub struct HttpCodec {
    // The underlying codec.
    codec: BytesCodec,
    // The associated node's span.
    span: Span,
    // The next kind of HTTP message expected.
    expecting: HttpMsg,
}

impl HttpCodec {
    pub fn new(span: Span, expecting: HttpMsg) -> Self {
        HttpCodec {
            codec: Default::default(),
            span,
            expecting,
        }
    }
}

impl Decoder for HttpCodec {
    type Item = BytesMut;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let mut raw_bytes = if let Some(bytes) = self.codec.decode(src)? {
            bytes
        } else {
            return Ok(None);
        };

        trace!(parent: &self.span, "got some raw bytes: {:?}", raw_bytes);

        let mut headers = [httparse::EMPTY_HEADER; 16];

        let res = match self.expecting {
            HttpMsg::Request => {
                let mut req = httparse::Request::new(&mut headers);
                req.parse(&raw_bytes)
            }
            HttpMsg::Response => {
                let mut resp = httparse::Response::new(&mut headers);
                resp.parse(&raw_bytes)
            }
        }
        .map_err(|e| {
            error!(parent: &self.span, "HTTP parse error: {}", e);
            io::ErrorKind::InvalidData
        })?;

        match res {
            httparse::Status::Partial => {
                // TODO: check if openssl ensures the completeness of requests
                warn!(parent: &self.span, "unexpected partial HTTP response");
                Ok(None)
            }
            httparse::Status::Complete(header_length) => {
                raw_bytes.advance(header_length);

                Ok(Some(raw_bytes))
            }
        }
    }
}

impl Encoder<Bytes> for HttpCodec {
    type Error = io::Error;

    fn encode(&mut self, message: Bytes, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.codec.encode(message, dst)
    }
}
