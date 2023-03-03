/// The Ripple handshake implementation.
use std::{io, pin::Pin};

use base64::{engine::general_purpose::STANDARD, Engine};
use bytes::Bytes;
use futures_util::{sink::SinkExt, TryStreamExt};
use openssl::ssl::Ssl;
use pea2pea::{protocols::Handshake, Connection, ConnectionSide, Pea2Pea};
use rand::{thread_rng, Rng};
use sha2::{Digest, Sha512};
use tokio_openssl::SslStream;
use tokio_util::codec::Framed;
use tracing::*;

use crate::{
    protocol::codecs::http::{HttpCodec, HttpMsg},
    tools::inner_node::{Crypto, InnerNode},
};

// Default handshake header values.
const CONNECTION: &str = "Upgrade";
const UPGRADE_REQ: &str = "XRPL/2.0, XRPL/2.1, XRPL/2.2"; // TODO: which ones should we handle?
const UPGRADE_RSP: &str = "XRPL/2.2";
const CONNECT_AS: &str = "Peer";
// txrr - enables transaction relay
// ledgerreplay - enables ledger replay
const X_PROTOCOL_CTL: &str = "txrr=1;ledgerreplay=1";

#[repr(u8)]
enum NodeType {
    Public = 28,
    #[allow(dead_code)]
    Private = 32,
}

/// Handshake configuration allows some customization of the handshake procedure.
#[derive(Clone)]
pub struct HandshakeCfg {
    /// Will flip a random bit in a random byte of shared value used for session signing.
    pub bitflip_shared_val: bool,

    /// Will flip a random bit in a random byte of the public key.
    pub bitflip_pub_key: bool,

    /// Identification header to be set during a handshake.
    /// Either 'User-Agent' or 'Server' depending on connection side.
    pub http_ident: String,

    /// A handshake field for the connection type.
    pub http_connection: String,

    /// A handshake field for the connection upgrade field - available versions sent
    /// in the handshake request.
    pub http_upgrade_req: String,

    /// A handshake field for the connection upgrade field - a chosen version sent in
    /// the handshake response.
    pub http_upgrade_rsp: String,

    /// A handshake field for the connector name.
    pub http_connect_as: String,

    /// A handshake field for the protocol CTL.
    pub http_x_protocol_ctl: String,
}

impl Default for HandshakeCfg {
    fn default() -> Self {
        Self {
            // Handshake procedure options.
            bitflip_shared_val: false,
            bitflip_pub_key: false,

            // Handshake HTTP fields.
            http_ident: "rippled-1.9.4".into(),
            http_connection: CONNECTION.to_owned(),
            http_upgrade_req: UPGRADE_REQ.to_owned(),
            http_upgrade_rsp: UPGRADE_RSP.to_owned(),
            http_connect_as: CONNECT_AS.to_owned(),
            http_x_protocol_ctl: X_PROTOCOL_CTL.to_owned(),
        }
    }
}

// Used to populate the Public-Key field.
fn encode_base58(node_type: NodeType, public_key: &[u8]) -> String {
    let mut payload = Vec::with_capacity(1 + public_key.len());

    payload.push(node_type as u8);
    payload.extend_from_slice(public_key);

    bs58::encode(payload)
        .with_alphabet(bs58::Alphabet::RIPPLE)
        .with_check()
        .into_string()
}

// Used to populate the Session-Signature field.
fn create_session_signature(crypto: &Crypto, shared_value: &[u8]) -> String {
    let message = secp256k1::Message::from_slice(shared_value).unwrap();
    let signature = crypto.engine.sign_ecdsa(&message, &crypto.private_key);
    let serialized = signature.serialize_der();

    STANDARD.encode(serialized)
}

// Used as input for create_session_signature.
fn get_shared_value<S>(tls_stream: &SslStream<S>) -> io::Result<Vec<u8>> {
    const MAX_FINISHED_SIZE: usize = 64;

    let mut finished = [0u8; MAX_FINISHED_SIZE];
    let finished_size = tls_stream.ssl().finished(&mut finished);
    let mut hasher = Sha512::new();
    hasher.update(&finished[..finished_size]);
    let finished_hash = hasher.finalize();

    let mut peer_finished = [0u8; MAX_FINISHED_SIZE];
    let peer_finished_size = tls_stream.ssl().peer_finished(&mut peer_finished);
    let mut hasher = Sha512::new();
    hasher.update(&peer_finished[..peer_finished_size]);
    let peer_finished_hash = hasher.finalize();

    let mut anded = [0u8; 64];
    for i in 0..64 {
        anded[i] = finished_hash[i] ^ peer_finished_hash[i];
    }

    let mut hasher = Sha512::new();
    hasher.update(anded);
    let hash = hasher.finalize()[..32].to_vec(); // the hash gets halved

    Ok(hash)
}

#[async_trait::async_trait]
impl Handshake for InnerNode {
    async fn perform_handshake(&self, mut conn: Connection) -> io::Result<Connection> {
        let own_conn_side = !conn.side();
        let stream = self.take_stream(&mut conn);
        let addr = conn.addr();

        // The function shouldn't be called in case the handshake config is not set.
        let hs_cfg = self
            .handshake_cfg
            .as_ref()
            .expect("a handshake config is not set");

        let tls_stream = match own_conn_side {
            ConnectionSide::Initiator => {
                let ssl = self
                    .tls
                    .connector
                    .configure()
                    .unwrap()
                    .into_ssl("domain") // is SNI and hostname verification enabled?
                    .unwrap();
                let mut tls_stream = SslStream::new(ssl, stream).unwrap();

                Pin::new(&mut tls_stream).connect().await.map_err(|e| {
                    error!(parent: self.node().span(), "TLS handshake error: {e}");
                    io::ErrorKind::InvalidData
                })?;

                // get the shared value based on the TLS handshake
                let mut shared_value = get_shared_value(&tls_stream)?;

                let public_key = &mut self.crypto.public_key.serialize().clone();
                // introduce intentional errors into handshake if needed
                if hs_cfg.bitflip_shared_val {
                    randomly_flip_bit(&mut shared_value);
                }
                if hs_cfg.bitflip_pub_key {
                    randomly_flip_bit(public_key.as_mut_slice());
                }

                // base58-encode the public key and create the session signature
                let base58_pk = encode_base58(NodeType::Public, public_key);
                let sig = create_session_signature(&self.crypto, &shared_value);

                // prepare the HTTP request message
                let mut req = Vec::new();
                let mut req_header = |mut header: String| {
                    // Append `\r\n' to every header.
                    header.push_str("\r\n");
                    req.extend_from_slice(header.as_bytes());
                };

                req_header("GET / HTTP/1.1".into());
                req_header(format!("User-Agent: {}", hs_cfg.http_ident));
                req_header(format!("Upgrade: {}", hs_cfg.http_upgrade_req));
                req_header(format!("Connection: {}", hs_cfg.http_connection));
                req_header(format!("Connect-As: {}", hs_cfg.http_connect_as));
                // TODO: an optional crawl goes here
                req_header(format!("X-Protocol-Ctl: {}", hs_cfg.http_x_protocol_ctl));
                // TODO: an optional network time goes here
                req_header(format!("Public-Key: {base58_pk}"));
                req_header(format!("Session-Signature: {sig}"));
                // TODO: an optional closed ledger goes here
                // TODO: an optional prev ledger goes here
                req_header("".into()); // An HTTP header ends with '\r\n'

                // use the HTTP codec to read/write the (post-TLS) handshake messages
                let req = Bytes::from(req);
                let codec = HttpCodec::new(self.node().span().clone(), HttpMsg::Response);
                let mut framed = Framed::new(&mut tls_stream, codec);

                // send the handshake HTTP request message
                trace!(parent: self.node().span(), "sending a request to {addr}: {req:?}");
                framed.send(req).await?;

                // read the HTTP request message (there should only be headers)
                let _ = framed.try_next().await?.ok_or(io::ErrorKind::InvalidData)?;

                tls_stream
            }
            ConnectionSide::Responder => {
                let ssl = Ssl::new(self.tls.acceptor.context()).unwrap();
                let mut tls_stream = SslStream::new(ssl, stream).unwrap();

                Pin::new(&mut tls_stream).accept().await.map_err(|e| {
                    error!(parent: self.node().span(), "TLS handshake error: {e}");
                    io::ErrorKind::InvalidData
                })?;

                // get the shared value based on the TLS handshake
                let mut shared_value = get_shared_value(&tls_stream)?;

                // use the HTTP codec to read/write the (post-TLS) handshake messages
                let codec = HttpCodec::new(self.node().span().clone(), HttpMsg::Request);
                let mut framed = Framed::new(&mut tls_stream, codec);

                // read the HTTP request message (there should only be headers)
                let request_body = framed.try_next().await?.ok_or(io::ErrorKind::InvalidData)?;
                if !request_body.is_empty() {
                    warn!(parent: self.node().span(), "trailing bytes in the handshake request from {addr}: {request_body:?}");
                }

                let public_key = &mut self.crypto.public_key.serialize().clone();
                // introduce intentional errors into handshake if needed
                if hs_cfg.bitflip_shared_val {
                    randomly_flip_bit(&mut shared_value);
                }
                if hs_cfg.bitflip_pub_key {
                    randomly_flip_bit(public_key.as_mut_slice());
                }
                // base58-encode the public key and create the session signature
                let base58_pk = encode_base58(NodeType::Public, public_key);
                let sig = create_session_signature(&self.crypto, &shared_value);

                // prepare the response
                let mut rsp = Vec::new();
                let mut rsp_header = |mut header: String| {
                    header.push_str("\r\n");
                    rsp.extend_from_slice(header.as_bytes());
                };

                rsp_header("HTTP/1.1 101 Switching Protocols".into());
                rsp_header(format!("Connection: {}", hs_cfg.http_connection));
                rsp_header(format!("Upgrade: {}", hs_cfg.http_upgrade_rsp));
                rsp_header(format!("Connect-As: {}", hs_cfg.http_connect_as));
                rsp_header(format!("Server: {}", hs_cfg.http_ident));
                // TODO: an optional crawl goes here
                rsp_header(format!("X-Protocol-Ctl: {}", hs_cfg.http_x_protocol_ctl));
                // TODO: an optional network time goes here
                rsp_header(format!("Public-Key: {base58_pk}"));
                rsp_header(format!("Session-Signature: {sig}"));
                // TODO: an optional closed ledger goes here
                // TODO: an optional prev ledger goes here
                rsp_header("".into());

                // send the handshake HTTP response message
                let rsp = Bytes::from(rsp);
                trace!(parent: self.node().span(), "responding to {addr} with {rsp:?}");
                framed.send(rsp).await?;

                tls_stream
            }
        };

        self.return_stream(&mut conn, tls_stream);

        Ok(conn)
    }
}

fn randomly_flip_bit(arr: &mut [u8]) {
    let idx = thread_rng().gen_range(0..arr.len());
    arr[idx] ^= 1 << thread_rng().gen_range(0..8);
}
