//! Based on https://github.com/sfackler/rust-openssl/blob/master/openssl/examples/mk_certs.rs.

use openssl::{
    asn1::Asn1Time,
    error::ErrorStack,
    hash::MessageDigest,
    pkey::{PKey, PKeyRef, Private},
    rsa::Rsa,
    x509::X509,
};

/// Make a CA certificate and private key
pub fn mk_ca_cert() -> Result<(X509, PKey<Private>), ErrorStack> {
    let rsa = Rsa::generate(2048)?;
    let key_pair = PKey::from_rsa(rsa)?;

    let mut cert_builder = X509::builder()?;
    cert_builder.set_pubkey(&key_pair)?;

    cert_builder.sign(&key_pair, MessageDigest::sha256())?;
    let cert = cert_builder.build();

    Ok((cert, key_pair))
}

/// Make a certificate and private key signed by the given CA cert and private key
pub fn mk_ca_signed_cert(
    ca_key_pair: &PKeyRef<Private>,
) -> Result<(X509, PKey<Private>), ErrorStack> {
    let rsa = Rsa::generate(2048)?;
    let key_pair = PKey::from_rsa(rsa)?;

    let mut cert_builder = X509::builder()?;
    cert_builder.set_pubkey(&key_pair)?;
    let not_before = Asn1Time::days_from_now(0)?;
    cert_builder.set_not_before(&not_before)?;
    let not_after = Asn1Time::days_from_now(365)?;
    cert_builder.set_not_after(&not_after)?;

    cert_builder.sign(ca_key_pair, MessageDigest::sha256())?;
    let cert = cert_builder.build();

    Ok((cert, key_pair))
}
