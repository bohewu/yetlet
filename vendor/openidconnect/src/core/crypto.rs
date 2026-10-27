use crate::core::jwk::CoreJsonCurveType;
use crate::core::{CoreJsonWebKey, CoreJsonWebKeyType};
use crate::helpers::Base64UrlEncodedBytes;
use crate::{JsonWebKey, SignatureVerificationError};

use std::ops::Deref;

fn rsa_public_key(
    key: &CoreJsonWebKey,
) -> Result<(&Base64UrlEncodedBytes, &Base64UrlEncodedBytes), String> {
    if *key.key_type() != CoreJsonWebKeyType::RSA {
        Err("RSA key required".to_string())
    } else {
        let n = key
            .n
            .as_ref()
            .ok_or_else(|| "RSA modulus `n` is missing".to_string())?;
        let e = key
            .e
            .as_ref()
            .ok_or_else(|| "RSA exponent `e` is missing".to_string())?;
        Ok((n, e))
    }
}

fn ec_public_key(
    key: &CoreJsonWebKey,
) -> Result<
    (
        &Base64UrlEncodedBytes,
        &Base64UrlEncodedBytes,
        &CoreJsonCurveType,
    ),
    String,
> {
    if *key.key_type() != CoreJsonWebKeyType::EllipticCurve {
        Err("EC key required".to_string())
    } else {
        let x = key
            .x
            .as_ref()
            .ok_or_else(|| "EC `x` part is missing".to_string())?;
        let y = key
            .y
            .as_ref()
            .ok_or_else(|| "EC `y` part is missing".to_string())?;
        let crv = key
            .crv
            .as_ref()
            .ok_or_else(|| "EC `crv` part is missing".to_string())?;
        Ok((x, y, crv))
    }
}

fn ed_public_key(
    key: &CoreJsonWebKey,
) -> Result<(&Base64UrlEncodedBytes, &CoreJsonCurveType), String> {
    if *key.key_type() != CoreJsonWebKeyType::OctetKeyPair {
        Err("OKP key required".to_string())
    } else {
        let x = key
            .x
            .as_ref()
            .ok_or_else(|| "OKP `x` part is missing".to_string())?;
        let crv = key
            .crv
            .as_ref()
            .ok_or_else(|| "OKP `crv` part is missing".to_string())?;
        Ok((x, crv))
    }
}

pub fn verify_rsa_signature(
    key: &CoreJsonWebKey,
    padding: impl rsa::traits::SignatureScheme,
    msg: &[u8],
    signature: &[u8],
) -> Result<(), SignatureVerificationError> {
    let (n, e) = rsa_public_key(key).map_err(SignatureVerificationError::InvalidKey)?;
    // let's n and e as a big integers to prevent issues with leading zeros
    // according to https://datatracker.ietf.org/doc/html/rfc7518#section-6.3.1.1
    // `n` is always unsigned (hence has sign plus)

    let n_bigint = rsa::BigUint::from_bytes_be(n.deref());
    let e_bigint = rsa::BigUint::from_bytes_be(e.deref());
    let public_key = rsa::RsaPublicKey::new(n_bigint, e_bigint)
        .map_err(|e| SignatureVerificationError::InvalidKey(e.to_string()))?;

    public_key
        .verify(padding, msg, signature)
        .map_err(|_| SignatureVerificationError::CryptoError("bad signature".to_string()))
}
/// According to RFC5480, Section-2.2 implementations of Elliptic Curve Cryptography MUST support the uncompressed form.
/// The first octet of the octet string indicates whether the uncompressed or compressed form is used. For the uncompressed
/// form, the first octet has to be 0x04.
/// According to https://briansmith.org/rustdoc/ring/signature/index.html#ecdsa__fixed-details-fixed-length-pkcs11-style-ecdsa-signatures,
/// to recover the X and Y coordinates from an octet string, the Octet-String-To-Elliptic-Curve-Point Conversion
/// is used (Section 2.3.4 of https://www.secg.org/sec1-v2.pdf).

pub fn verify_ec_signature(
    key: &CoreJsonWebKey,
    msg: &[u8],
    signature: &[u8],
) -> Result<(), SignatureVerificationError> {
    use p256::ecdsa::signature::Verifier;

    let (x, y, crv) = ec_public_key(key).map_err(SignatureVerificationError::InvalidKey)?;
    let mut pk = vec![0x04];
    pk.extend(x.deref());
    pk.extend(y.deref());
    match *crv {
        CoreJsonCurveType::P256 => {
            let public_key = p256::ecdsa::VerifyingKey::from_sec1_bytes(&pk)
                .map_err(|e| SignatureVerificationError::InvalidKey(e.to_string()))?;
            public_key
                .verify(
                    msg,
                    &p256::ecdsa::Signature::from_slice(signature).map_err(|_| {
                        SignatureVerificationError::CryptoError("Invalid signature".to_string())
                    })?,
                )
                .map_err(|_| {
                    SignatureVerificationError::CryptoError("EC Signature was wrong".to_string())
                })
        }
        CoreJsonCurveType::P384 => {
            let public_key = p384::ecdsa::VerifyingKey::from_sec1_bytes(&pk)
                .map_err(|e| SignatureVerificationError::InvalidKey(e.to_string()))?;
            public_key
                .verify(
                    msg,
                    &p384::ecdsa::Signature::from_slice(signature).map_err(|_| {
                        SignatureVerificationError::CryptoError("Invalid signature".to_string())
                    })?,
                )
                .map_err(|_| {
                    SignatureVerificationError::CryptoError("EC Signature was wrong".to_string())
                })
        }
        CoreJsonCurveType::P521 => Err(SignatureVerificationError::UnsupportedAlg(
            "P521".to_string(),
        )),
        _ => Err(SignatureVerificationError::InvalidKey(format!(
            "unrecognized curve `{crv:?}`"
        ))),
    }
}

pub fn verify_ed_signature(
    key: &CoreJsonWebKey,
    msg: &[u8],
    signature: &[u8],
) -> Result<(), SignatureVerificationError> {
    use ed25519_dalek::Verifier;

    let (x, crv) = ed_public_key(key).map_err(SignatureVerificationError::InvalidKey)?;

    match *crv {
        CoreJsonCurveType::Ed25519 => {
            let public_key = ed25519_dalek::VerifyingKey::try_from(x.deref().as_slice())
                .map_err(|e| SignatureVerificationError::InvalidKey(e.to_string()))?;

            public_key
                .verify(
                    msg,
                    &ed25519_dalek::Signature::from_slice(signature).map_err(|_| {
                        SignatureVerificationError::CryptoError("invalid signature".to_string())
                    })?,
                )
                .map_err(|_| {
                    SignatureVerificationError::CryptoError("incorrect EdDSA signature".to_string())
                })
        }
        _ => Err(SignatureVerificationError::InvalidKey(format!(
            "unrecognized curve `{crv:?}`"
        ))),
    }
}
