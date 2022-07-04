use std::{net::IpAddr, sync::Arc};

use openssl::{
    pkcs12::Pkcs12,
    ssl::{SslAcceptor, SslConnector, SslMethod, SslVerifyMode},
};
use pea2pea::{Node, Pea2Pea};
use secp256k1::{PublicKey, Secp256k1, SecretKey};

/// Enables tracing for all [`SyntheticNode`] instances (usually scoped by test).
pub fn enable_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};

    fmt()
        .with_test_writer()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}

// A synthetic node adhering to Ripple's network protocol.
#[derive(Clone)]
pub struct SyntheticNode {
    node: Node,
    pub crypto: Arc<Crypto>,
    pub tls: Tls,
}

// An object cointaining TLS handlers.
#[derive(Clone)]
pub struct Tls {
    pub acceptor: SslAcceptor,
    pub connector: SslConnector,
}

// An object dedicated to cryptographic functionalities.
pub struct Crypto {
    pub engine: Secp256k1<secp256k1::All>,
    pub private_key: SecretKey,
    pub public_key: PublicKey,
}

impl Pea2Pea for SyntheticNode {
    fn node(&self) -> &Node {
        &self.node
    }
}

impl SyntheticNode {
    pub async fn new(config: pea2pea::Config) -> Self {
        // generate the keypair and prepare the crypto engine

        let engine = Secp256k1::new();
        let (private_key, public_key) = engine.generate_keypair(&mut secp256k1::rand::thread_rng());
        let crypto = Arc::new(Crypto {
            engine,
            private_key,
            public_key,
        });

        // TLS acceptor

        let file = include_bytes!("./tls_identity.pfx");
        let identity = Pkcs12::from_der(&file[..]).unwrap().parse("pass").unwrap();
        let mut acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        acceptor.set_private_key(&identity.pkey).unwrap();
        acceptor.set_certificate(&identity.cert).unwrap();
        let acceptor = acceptor.build();

        // TLS connector

        let mut connector = SslConnector::builder(SslMethod::tls()).unwrap();
        connector.set_verify(SslVerifyMode::NONE); // we might remove it once the keypair is solid
        let connector = connector.build();

        // the node

        Self {
            node: Node::new(Some(config)).await.unwrap(),
            crypto,
            tls: Tls {
                acceptor,
                connector,
            },
        }
    }

    pub fn is_connected_ip(&self, ip: IpAddr) -> bool {
        self.node()
            .connected_addrs()
            .iter()
            .any(|addr| addr.ip() == ip)
    }
}
