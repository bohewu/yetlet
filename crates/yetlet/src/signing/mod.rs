//! Experimental opt-in native signing. Exact bytes, explicit trust, no network or file I/O.
//! Leaf pins are operator trust, not PKI chain validation or trusted signing time.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt, future::Future};

mod local;
pub use local::{
    GeneratedMaterial, LocalProvider, LocalSigner, PublicMaterial, generate_self_signed,
};

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyRef {
    pub provider: String,
    pub key: String,
    pub version: String,
}
impl KeyRef {
    fn validate(&self) -> Result<()> {
        if [&self.provider, &self.key, &self.version]
            .iter()
            .any(|v| v.is_empty())
            || self.version.eq_ignore_ascii_case("latest")
        {
            return Err(Error::PolicyRejected);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Sha256Digest(pub [u8; 32]);
impl Sha256Digest {
    pub fn of(bytes: &[u8]) -> Self {
        Self(openssl::sha::sha256(bytes))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Algorithm {
    EcdsaP256Sha256,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Envelope {
    pub version: u8,
    pub algorithm: Algorithm,
    pub key_ref: KeyRef,
    #[serde(with = "signature_serde")]
    pub signature: [u8; 64],
}
mod signature_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};
    pub fn serialize<S: Serializer>(v: &[u8; 64], s: S) -> Result<S::Ok, S::Error> {
        v.as_slice().serialize(s)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 64], D::Error> {
        Vec::<u8>::deserialize(d)?
            .try_into()
            .map_err(|_| D::Error::custom("expected 64 signature bytes"))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidMaterial,
    UnknownKey,
    UnsupportedAlgorithm,
    InvalidSignature,
    PolicyRejected,
    BackendUnavailable,
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidMaterial => "invalid signing material or password",
            Self::UnknownKey => "unknown key version",
            Self::UnsupportedAlgorithm => "unsupported signing algorithm",
            Self::InvalidSignature => "invalid signature",
            Self::PolicyRejected => "signing policy rejected the operation",
            Self::BackendUnavailable => "signing backend unavailable",
        })
    }
}
impl std::error::Error for Error {}
pub type Result<T> = std::result::Result<T, Error>;

pub struct BackendSignature {
    pub key_ref: KeyRef,
    pub signature: [u8; 64],
}
pub struct ResolvedPublicKey {
    pub key_ref: KeyRef,
    pub material: PublicMaterial,
}

pub trait SigningBackend: Send + Sync {
    fn sign_digest(
        &self,
        key: &KeyRef,
        digest: Sha256Digest,
    ) -> impl Future<Output = Result<BackendSignature>> + Send;
}
pub trait PublicKeySource: Send + Sync {
    fn public_key(&self, key: &KeyRef) -> impl Future<Output = Result<ResolvedPublicKey>> + Send;
}

/// An in-memory public-key source for certificate-backed or certificate-free material.
/// Contains no password or signing capability; trust is configured separately by [`Policy`].
#[derive(Clone)]
pub struct PublicKeySet(BTreeMap<KeyRef, PublicMaterial>);
impl PublicKeySet {
    pub fn new(entries: Vec<(KeyRef, PublicMaterial)>) -> Result<Self> {
        let mut map = BTreeMap::new();
        for (key, material) in entries {
            key.validate()?;
            if map.insert(key, material).is_some() {
                return Err(Error::PolicyRejected);
            }
        }
        Ok(Self(map))
    }
}
impl PublicKeySource for PublicKeySet {
    async fn public_key(&self, key: &KeyRef) -> Result<ResolvedPublicKey> {
        Ok(ResolvedPublicKey {
            key_ref: key.clone(),
            material: self.0.get(key).ok_or(Error::UnknownKey)?.clone(),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyState {
    Active,
    Retired,
    Revoked,
}

pub struct TrustedKey {
    key_ref: KeyRef,
    material: PublicMaterial,
    state: KeyState,
}
impl TrustedKey {
    /// The supplied pin must come from operator configuration, never the envelope.
    /// Expired history remains loadable; each operation checks current validity.
    pub fn certificate(
        key_ref: KeyRef,
        pem: &[u8],
        expected_pin: [u8; 32],
        state: KeyState,
    ) -> Result<Self> {
        let material = PublicMaterial::from_certificate_pem(pem)?;
        if material.certificate_pin() != Some(expected_pin) {
            return Err(Error::PolicyRejected);
        }
        Self::public_key(key_ref, material, state)
    }
    /// Explicitly trust provider public material, including certificate-free KMS SPKI.
    pub fn public_key(key_ref: KeyRef, material: PublicMaterial, state: KeyState) -> Result<Self> {
        key_ref.validate()?;
        Ok(Self {
            key_ref,
            material,
            state,
        })
    }
}

/// Retains all identities, including revoked keys, to prevent revocation bypass.
pub struct Policy {
    keys: BTreeMap<KeyRef, TrustedKey>,
}
impl Policy {
    pub fn new(keys: Vec<TrustedKey>) -> Result<Self> {
        let mut policy = Self {
            keys: BTreeMap::new(),
        };
        for key in keys {
            policy.insert(key)?;
        }
        Ok(policy)
    }
    pub fn insert(&mut self, key: TrustedKey) -> Result<()> {
        if self.keys.contains_key(&key.key_ref)
            || self.keys.values().any(|existing| {
                existing.material.same_public_key(&key.material)
                    || (existing.state == KeyState::Active
                        && key.state == KeyState::Active
                        && existing.key_ref.provider == key.key_ref.provider
                        && existing.key_ref.key == key.key_ref.key)
            })
        {
            return Err(Error::PolicyRejected);
        }
        self.keys.insert(key.key_ref.clone(), key);
        Ok(())
    }
    /// Activation retires the former active version of this logical key atomically.
    /// Revocation is terminal for this policy; retained identities cannot be removed.
    pub fn set_state(&mut self, key: &KeyRef, state: KeyState) -> Result<()> {
        let entry = self.keys.get(key).ok_or(Error::UnknownKey)?;
        if entry.state == KeyState::Revoked && state != KeyState::Revoked {
            return Err(Error::PolicyRejected);
        }
        if state == KeyState::Active {
            entry.material.check_usable()?;
            for existing in self.keys.values_mut() {
                if existing.key_ref.provider == key.provider
                    && existing.key_ref.key == key.key
                    && existing.state == KeyState::Active
                {
                    existing.state = KeyState::Retired;
                }
            }
        }
        self.keys.get_mut(key).ok_or(Error::UnknownKey)?.state = state;
        Ok(())
    }
    fn authorize(&self, key: &KeyRef, signing: bool) -> Result<&TrustedKey> {
        let trusted = self.keys.get(key).ok_or(Error::UnknownKey)?;
        if trusted.state == KeyState::Revoked || (signing && trusted.state != KeyState::Active) {
            return Err(Error::PolicyRejected);
        }
        trusted.material.check_usable()?;
        Ok(trusted)
    }
}

/// Verification needs only a public source and trust policy.
pub struct Verifier<P> {
    public: P,
    policy: Policy,
}
impl<P: PublicKeySource> Verifier<P> {
    pub fn new(public: P, policy: Policy) -> Self {
        Self { public, policy }
    }
    pub fn policy_mut(&mut self) -> &mut Policy {
        &mut self.policy
    }
    pub async fn verify(&self, bytes: &[u8], envelope: &Envelope) -> Result<()> {
        if envelope.version != 1 {
            return Err(Error::UnsupportedAlgorithm);
        }
        let trusted = self.policy.authorize(&envelope.key_ref, false)?;
        let resolved = self.public.public_key(&envelope.key_ref).await?;
        if resolved.key_ref != envelope.key_ref || !trusted.material.matches(&resolved.material) {
            return Err(Error::PolicyRejected);
        }
        resolved.material.check_usable()?;
        resolved
            .material
            .verify_digest(Sha256Digest::of(bytes), &envelope.signature)
    }
}

pub struct SigningService<B, P> {
    backend: B,
    verifier: Verifier<P>,
}
impl<B: SigningBackend, P: PublicKeySource> SigningService<B, P> {
    pub fn new(backend: B, public: P, policy: Policy) -> Self {
        Self {
            backend,
            verifier: Verifier::new(public, policy),
        }
    }
    pub fn policy_mut(&mut self) -> &mut Policy {
        self.verifier.policy_mut()
    }
    pub async fn sign(&self, key: &KeyRef, bytes: &[u8]) -> Result<Envelope> {
        // Borrowed policy is immutable for the admitted operation across await.
        let trusted = self.verifier.policy.authorize(key, true)?;
        let exact = key.clone();
        let digest = Sha256Digest::of(bytes);
        let signed = self.backend.sign_digest(&exact, digest).await?;
        if signed.key_ref != exact {
            return Err(Error::PolicyRejected);
        }
        trusted.material.verify_digest(digest, &signed.signature)?;
        Ok(Envelope {
            version: 1,
            algorithm: Algorithm::EcdsaP256Sha256,
            key_ref: exact,
            signature: signed.signature,
        })
    }
    pub async fn verify(&self, bytes: &[u8], envelope: &Envelope) -> Result<()> {
        self.verifier.verify(bytes, envelope).await
    }
}
