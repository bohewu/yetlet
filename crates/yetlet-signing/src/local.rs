use crate::{
    BackendSignature, Error, KeyRef, PublicKeySource, ResolvedPublicKey, Result, Sha256Digest,
    SigningBackend,
};
use foreign_types::ForeignType;
use openssl::{
    asn1::Asn1Time,
    bn::{BigNum, MsbOption},
    ec::{EcGroup, EcKey},
    ecdsa::EcdsaSig,
    hash::MessageDigest,
    md::Md,
    nid::Nid,
    pkcs12::Pkcs12,
    pkey::{PKey, Private, Public},
    pkey_ctx::PkeyCtx,
    symm::Cipher,
    x509::{
        X509, X509NameBuilder,
        extension::{BasicConstraints, KeyUsage},
    },
};
use std::{cmp::Ordering, collections::BTreeMap};

// The maintained C API is not yet exposed by openssl-sys. It only queries
// MAC presence; all decoding and integrity verification remain in OpenSSL.
unsafe extern "C" {
    fn PKCS12_mac_present(p12: *const openssl_sys::PKCS12) -> std::ffi::c_int;
}

fn invalid(_: openssl::error::ErrorStack) -> Error {
    Error::InvalidMaterial
}
fn unavailable(_: openssl::error::ErrorStack) -> Error {
    Error::BackendUnavailable
}

/// Public-key data with an optional certificate and leaf pin.
/// Certificate dates, when present, are checked at operation time so revoked
/// or expired history can still participate in duplicate-key detection.
#[derive(Clone)]
pub struct PublicMaterial {
    key: PKey<Public>,
    certificate: Option<X509>,
    pin: Option<[u8; 32]>,
}
impl PublicMaterial {
    pub fn from_certificate_pem(pem: &[u8]) -> Result<Self> {
        let certificates = X509::stack_from_pem(pem).map_err(invalid)?;
        if certificates.len() != 1 {
            return Err(Error::InvalidMaterial);
        }
        Self::from_certificate(
            certificates
                .into_iter()
                .next()
                .ok_or(Error::InvalidMaterial)?,
        )
    }
    fn from_certificate(certificate: X509) -> Result<Self> {
        let key = certificate.public_key().map_err(invalid)?;
        Self::check_key(&key)?;
        // SAFETY: the owned X509 remains alive throughout both calls. OpenSSL
        // parses/caches extensions itself; no custom ASN.1 or text parsing.
        let flags = unsafe { openssl_sys::X509_get_extension_flags(certificate.as_ptr()) };
        let usage = unsafe { openssl_sys::X509_get_key_usage(certificate.as_ptr()) };
        if flags & openssl_sys::EXFLAG_INVALID != 0
            || flags & openssl_sys::EXFLAG_KUSAGE == 0
            || usage & openssl_sys::X509v3_KU_DIGITAL_SIGNATURE == 0
        {
            return Err(Error::PolicyRejected);
        }
        let pin = openssl::sha::sha256(&certificate.to_der().map_err(invalid)?);
        Ok(Self {
            key,
            certificate: Some(certificate),
            pin: Some(pin),
        })
    }
    /// Certificate-free public keys for separately implemented provider adapters.
    pub fn from_public_key_der(der: &[u8]) -> Result<Self> {
        let key = PKey::public_key_from_der(der).map_err(invalid)?;
        Self::check_key(&key)?;
        Ok(Self {
            key,
            certificate: None,
            pin: None,
        })
    }
    fn check_key(key: &PKey<Public>) -> Result<()> {
        let ec = key.ec_key().map_err(|_| Error::UnsupportedAlgorithm)?;
        if ec.group().curve_name() != Some(Nid::X9_62_PRIME256V1) {
            return Err(Error::UnsupportedAlgorithm);
        }
        ec.check_key().map_err(invalid)
    }
    pub fn check_usable(&self) -> Result<()> {
        if let Some(cert) = &self.certificate {
            let now = Asn1Time::days_from_now(0).map_err(unavailable)?;
            if cert.not_before().compare(&now).map_err(invalid)? == Ordering::Greater
                || cert.not_after().compare(&now).map_err(invalid)? != Ordering::Greater
            {
                return Err(Error::PolicyRejected);
            }
        }
        Ok(())
    }
    pub fn certificate_pin(&self) -> Option<[u8; 32]> {
        self.pin
    }
    pub fn certificate_pem(&self) -> Result<Vec<u8>> {
        self.certificate
            .as_ref()
            .ok_or(Error::InvalidMaterial)?
            .to_pem()
            .map_err(invalid)
    }
    pub fn certificate_validity(&self) -> Option<(String, String)> {
        self.certificate
            .as_ref()
            .map(|c| (c.not_before().to_string(), c.not_after().to_string()))
    }
    pub fn certificate_dates(&self) -> Result<(String, String)> {
        self.certificate_validity().ok_or(Error::InvalidMaterial)
    }
    pub fn public_key_der(&self) -> Result<Vec<u8>> {
        self.key.public_key_to_der().map_err(invalid)
    }
    pub fn same_public_key(&self, other: &Self) -> bool {
        self.key.public_eq(&other.key)
    }
    pub(crate) fn matches(&self, other: &Self) -> bool {
        self.pin == other.pin && self.same_public_key(other)
    }
    pub(crate) fn verify_digest(&self, digest: Sha256Digest, signature: &[u8; 64]) -> Result<()> {
        let r = BigNum::from_slice(&signature[..32]).map_err(|_| Error::InvalidSignature)?;
        let s = BigNum::from_slice(&signature[32..]).map_err(|_| Error::InvalidSignature)?;
        let der = EcdsaSig::from_private_components(r, s)
            .and_then(|s| s.to_der())
            .map_err(|_| Error::InvalidSignature)?;
        let mut ctx = PkeyCtx::new(&self.key).map_err(unavailable)?;
        ctx.verify_init().map_err(unavailable)?;
        ctx.set_signature_md(Md::sha256()).map_err(unavailable)?;
        if ctx
            .verify(&digest.0, &der)
            .map_err(|_| Error::InvalidSignature)?
        {
            Ok(())
        } else {
            Err(Error::InvalidSignature)
        }
    }
}

/// Holds a private key. Deliberately has no Debug/Serialize implementation.
pub struct LocalSigner {
    key_ref: KeyRef,
    private: PKey<Private>,
    public: PublicMaterial,
}
impl LocalSigner {
    pub fn from_pem(
        key_ref: KeyRef,
        encrypted_pem: &[u8],
        password: &str,
        certificate_pem: &[u8],
    ) -> Result<Self> {
        check_password(password)?;
        // Restrict the reference PEM profile to encrypted PKCS#8. Avoid OpenSSL's
        // interactive password callback and accidental plaintext acceptance.
        if !encrypted_pem
            .starts_with(concat!("-----BEGIN ", "ENCRYPTED PRIVATE KEY", "-----").as_bytes())
        {
            return Err(Error::InvalidMaterial);
        }
        let private = PKey::private_key_from_pem_passphrase(encrypted_pem, password.as_bytes())
            .map_err(invalid)?;
        let public = PublicMaterial::from_certificate_pem(certificate_pem)?;
        Self::new(key_ref, private, public)
    }
    pub fn from_pkcs12(key_ref: KeyRef, der: &[u8], password: &str) -> Result<Self> {
        check_password(password)?;
        let archive = Pkcs12::from_der(der).map_err(invalid)?;
        // SAFETY: archive owns a live PKCS12 object throughout this read-only call.
        if unsafe { PKCS12_mac_present(archive.as_ptr()) } != 1 {
            return Err(Error::InvalidMaterial);
        }
        // parse2 verifies the present MAC with this password before decoding bags.
        let parsed = archive.parse2(password).map_err(invalid)?;
        let private = parsed.pkey.ok_or(Error::InvalidMaterial)?;
        // Additional certificates are deliberately not trusted or chain-validated.
        let public = PublicMaterial::from_certificate(parsed.cert.ok_or(Error::InvalidMaterial)?)?;
        Self::new(key_ref, private, public)
    }
    fn new(key_ref: KeyRef, private: PKey<Private>, public: PublicMaterial) -> Result<Self> {
        key_ref.validate()?;
        if !private.public_eq(&public.key) {
            return Err(Error::InvalidMaterial);
        }
        private
            .ec_key()
            .map_err(invalid)?
            .check_key()
            .map_err(invalid)?;
        public.check_usable()?;
        Ok(Self {
            key_ref,
            private,
            public,
        })
    }
    pub fn public_material(&self) -> PublicMaterial {
        self.public.clone()
    }
    pub fn encrypted_private_key_pem(&self, password: &str) -> Result<Vec<u8>> {
        check_password(password)?;
        self.private
            .private_key_to_pem_pkcs8_passphrase(Cipher::aes_256_cbc(), password.as_bytes())
            .map_err(unavailable)
    }
    fn sign(&self, key: &KeyRef, digest: Sha256Digest) -> Result<BackendSignature> {
        if key != &self.key_ref {
            return Err(Error::UnknownKey);
        }
        self.public.check_usable()?;
        let mut ctx = PkeyCtx::new(&self.private).map_err(unavailable)?;
        ctx.sign_init().map_err(unavailable)?;
        ctx.set_signature_md(Md::sha256()).map_err(unavailable)?;
        let mut der = Vec::new();
        ctx.sign_to_vec(&digest.0, &mut der).map_err(unavailable)?;
        let signature = EcdsaSig::from_der(&der).map_err(unavailable)?;
        let mut raw = [0; 64];
        raw[..32].copy_from_slice(&signature.r().to_vec_padded(32).map_err(unavailable)?);
        raw[32..].copy_from_slice(&signature.s().to_vec_padded(32).map_err(unavailable)?);
        Ok(BackendSignature {
            key_ref: self.key_ref.clone(),
            signature: raw,
        })
    }
}
impl SigningBackend for LocalSigner {
    async fn sign_digest(&self, key: &KeyRef, digest: Sha256Digest) -> Result<BackendSignature> {
        self.sign(key, digest)
    }
}
impl PublicKeySource for LocalSigner {
    async fn public_key(&self, key: &KeyRef) -> Result<ResolvedPublicKey> {
        if key != &self.key_ref {
            return Err(Error::UnknownKey);
        }
        Ok(ResolvedPublicKey {
            key_ref: key.clone(),
            material: self.public.clone(),
        })
    }
}

/// A simple configured collection; no dynamic provider registry or file discovery.
pub struct LocalProvider(BTreeMap<KeyRef, LocalSigner>);
impl LocalProvider {
    pub fn new(signers: Vec<LocalSigner>) -> Result<Self> {
        let mut map = BTreeMap::new();
        for signer in signers {
            if map.insert(signer.key_ref.clone(), signer).is_some() {
                return Err(Error::PolicyRejected);
            }
        }
        Ok(Self(map))
    }
}
impl SigningBackend for LocalProvider {
    async fn sign_digest(&self, key: &KeyRef, digest: Sha256Digest) -> Result<BackendSignature> {
        self.0.get(key).ok_or(Error::UnknownKey)?.sign(key, digest)
    }
}
impl PublicKeySource for LocalProvider {
    async fn public_key(&self, key: &KeyRef) -> Result<ResolvedPublicKey> {
        self.0
            .get(key)
            .ok_or(Error::UnknownKey)?
            .public_key(key)
            .await
    }
}

/// All private material returned here is encrypted. No Debug or Serialize.
pub struct GeneratedMaterial {
    pub encrypted_private_key_pem: Vec<u8>,
    pub certificate_pem: Vec<u8>,
    pub pkcs12_der: Vec<u8>,
    pub certificate_pin: [u8; 32],
}
fn check_password(password: &str) -> Result<()> {
    if password.is_empty() || password.as_bytes().contains(&0) {
        return Err(Error::InvalidMaterial);
    }
    Ok(())
}
/// Generate disposable local self-signed material. Never a CA certificate renewal.
pub fn generate_self_signed(
    common_name: &str,
    password: &str,
    days: u32,
) -> Result<GeneratedMaterial> {
    check_password(password)?;
    if common_name.is_empty()
        || common_name.contains('\0')
        || common_name.len() > 64
        || days == 0
        || days > 3650
    {
        return Err(Error::InvalidMaterial);
    }
    let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1).map_err(unavailable)?;
    let private =
        PKey::from_ec_key(EcKey::generate(&group).map_err(unavailable)?).map_err(unavailable)?;
    let mut name = X509NameBuilder::new().map_err(unavailable)?;
    name.append_entry_by_nid(Nid::COMMONNAME, common_name)
        .map_err(unavailable)?;
    let name = name.build();
    let mut builder = X509::builder().map_err(unavailable)?;
    builder.set_version(2).map_err(unavailable)?;
    let mut serial = BigNum::new().map_err(unavailable)?;
    serial
        .rand(128, MsbOption::ONE, false)
        .map_err(unavailable)?;
    let serial = serial.to_asn1_integer().map_err(unavailable)?;
    builder.set_serial_number(&serial).map_err(unavailable)?;
    builder.set_subject_name(&name).map_err(unavailable)?;
    builder.set_issuer_name(&name).map_err(unavailable)?;
    builder.set_pubkey(&private).map_err(unavailable)?;
    let start = Asn1Time::days_from_now(0).map_err(unavailable)?;
    let end = Asn1Time::days_from_now(days).map_err(unavailable)?;
    builder.set_not_before(&start).map_err(unavailable)?;
    builder.set_not_after(&end).map_err(unavailable)?;
    builder
        .append_extension(
            BasicConstraints::new()
                .critical()
                .build()
                .map_err(unavailable)?,
        )
        .map_err(unavailable)?;
    builder
        .append_extension(
            KeyUsage::new()
                .critical()
                .digital_signature()
                .build()
                .map_err(unavailable)?,
        )
        .map_err(unavailable)?;
    builder
        .sign(&private, MessageDigest::sha256())
        .map_err(unavailable)?;
    let cert = builder.build();
    let certificate_pin = openssl::sha::sha256(&cert.to_der().map_err(unavailable)?);
    let mut pfx = Pkcs12::builder();
    pfx.name(common_name).pkey(&private).cert(&cert);
    Ok(GeneratedMaterial {
        encrypted_private_key_pem: private
            .private_key_to_pem_pkcs8_passphrase(Cipher::aes_256_cbc(), password.as_bytes())
            .map_err(unavailable)?,
        certificate_pem: cert.to_pem().map_err(unavailable)?,
        pkcs12_der: pfx
            .build2(password)
            .and_then(|p| p.to_der())
            .map_err(unavailable)?,
        certificate_pin,
    })
}
