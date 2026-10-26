use futures::executor::block_on;
use openssl::{
    asn1::Asn1Time,
    bn::BigNum,
    ec::{EcGroup, EcKey},
    hash::MessageDigest,
    nid::Nid,
    pkey::{PKey, Private},
    x509::{X509, extension::KeyUsage},
};
use yetlet_signing::{
    BackendSignature, Envelope, Error, GeneratedMaterial, KeyRef, KeyState, LocalProvider,
    LocalSigner, Policy, PublicKeySet, PublicKeySource, PublicMaterial, ResolvedPublicKey, Result,
    Sha256Digest, SigningBackend, SigningService, TrustedKey, Verifier, generate_self_signed,
};

const PASSWORD: &str = "disposable synthetic password";
fn key(version: &str) -> KeyRef {
    KeyRef {
        provider: "local".into(),
        key: "documents".into(),
        version: version.into(),
    }
}
fn material() -> GeneratedMaterial {
    generate_self_signed("Disposable test identity", PASSWORD, 2).unwrap()
}
fn signer(id: KeyRef, m: &GeneratedMaterial) -> LocalSigner {
    LocalSigner::from_pem(
        id,
        &m.encrypted_private_key_pem,
        PASSWORD,
        &m.certificate_pem,
    )
    .unwrap()
}
fn trusted(id: KeyRef, m: &GeneratedMaterial, state: KeyState) -> TrustedKey {
    TrustedKey::certificate(id, &m.certificate_pem, m.certificate_pin, state).unwrap()
}
fn service(m: &GeneratedMaterial) -> SigningService<LocalProvider, PublicKeySet> {
    let local = signer(key("v1"), m);
    let public = PublicKeySet::new(vec![(key("v1"), local.public_material())]).unwrap();
    SigningService::new(
        LocalProvider::new(vec![local]).unwrap(),
        public,
        Policy::new(vec![trusted(key("v1"), m, KeyState::Active)]).unwrap(),
    )
}

#[test]
fn native_supported_library() {
    assert!(
        openssl::version::version().starts_with("OpenSSL 3.5.9"),
        "qualification requires OpenSSL 3.5.9; found {}",
        openssl::version::version()
    );
}

#[test]
fn exact_bytes_tamper_fixed_width_and_public_only_verification() {
    let m = material();
    let service = service(&m);
    let bytes = b"exact\r\nbytes\0\xff";
    let envelope = block_on(service.sign(&key("v1"), bytes)).unwrap();
    assert_eq!(envelope.signature.len(), 64);
    let public = PublicKeySet::new(vec![(
        key("v1"),
        PublicMaterial::from_certificate_pem(&m.certificate_pem).unwrap(),
    )])
    .unwrap();
    let resolved = block_on(public.clone().public_key(&key("v1"))).unwrap();
    assert_eq!(resolved.key_ref, key("v1"));
    assert_eq!(resolved.material.certificate_pin(), Some(m.certificate_pin));
    let verifier = Verifier::new(
        public,
        Policy::new(vec![trusted(key("v1"), &m, KeyState::Retired)]).unwrap(),
    );
    drop(service); // The verifier owns neither private material nor a signer.
    block_on(verifier.verify(bytes, &envelope)).unwrap();
    assert_eq!(
        block_on(verifier.verify(b"exact\nbytes\0\xff", &envelope)),
        Err(Error::InvalidSignature)
    );
    let mut changed = envelope.clone();
    changed.signature[0] ^= 1;
    assert_eq!(
        block_on(verifier.verify(bytes, &changed)),
        Err(Error::InvalidSignature)
    );
    changed = envelope.clone();
    changed.version = 2;
    assert_eq!(
        block_on(verifier.verify(bytes, &changed)),
        Err(Error::UnsupportedAlgorithm)
    );
    changed = envelope.clone();
    changed.key_ref.provider = "attacker-provider".into();
    assert_eq!(
        block_on(verifier.verify(bytes, &changed)),
        Err(Error::UnknownKey)
    );
    let json = serde_json::to_value(&envelope).unwrap();
    assert_eq!(
        serde_json::from_value::<Envelope>(json.clone()).unwrap(),
        envelope
    );
    let mut malformed = json.clone();
    malformed["signature"] = serde_json::json!([1, 2]);
    assert!(serde_json::from_value::<Envelope>(malformed).is_err());
    let mut malformed = json;
    malformed["algorithm"] = serde_json::json!("RSA");
    assert!(serde_json::from_value::<Envelope>(malformed).is_err());
}

#[test]
fn pem_pfx_same_key_and_two_independent_providers() {
    let m = material();
    let pem = signer(key("v1"), &m);
    let pfx = LocalSigner::from_pkcs12(key("v1"), &m.pkcs12_der, PASSWORD).unwrap();
    assert!(
        pem.public_material()
            .same_public_key(&pfx.public_material())
    );
    let digest = Sha256Digest::of(b"already hashed once");
    let signature = block_on(pfx.sign_digest(&key("v1"), digest)).unwrap();
    // Verify externally with a message-hashing EVP verifier: a second backend
    // hash would fail this independent operation.
    let cert = X509::from_pem(&m.certificate_pem).unwrap();
    let public_key = cert.public_key().unwrap();
    let mut verifier = openssl::sign::Verifier::new(MessageDigest::sha256(), &public_key).unwrap();
    let der = openssl::ecdsa::EcdsaSig::from_private_components(
        BigNum::from_slice(&signature.signature[..32]).unwrap(),
        BigNum::from_slice(&signature.signature[32..]).unwrap(),
    )
    .unwrap()
    .to_der()
    .unwrap();
    assert!(
        verifier
            .verify_oneshot(&der, b"already hashed once")
            .unwrap()
    );
    for local in [pem, pfx] {
        let public = PublicKeySet::new(vec![(key("v1"), local.public_material())]).unwrap();
        let service = SigningService::new(
            local,
            public,
            Policy::new(vec![trusted(key("v1"), &m, KeyState::Active)]).unwrap(),
        );
        let signed = block_on(service.sign(&key("v1"), b"provider contract")).unwrap();
        block_on(service.verify(b"provider contract", &signed)).unwrap();
    }
}

#[test]
fn password_malformed_missing_private_and_mismatch_are_closed() {
    let m = material();
    let other = material();
    for password in ["wrong password", "", "embedded\0nul"] {
        assert!(matches!(
            LocalSigner::from_pem(
                key("v1"),
                &m.encrypted_private_key_pem,
                password,
                &m.certificate_pem
            ),
            Err(Error::InvalidMaterial)
        ));
        assert!(matches!(
            LocalSigner::from_pkcs12(key("v1"), &m.pkcs12_der, password),
            Err(Error::InvalidMaterial)
        ));
    }
    assert!(matches!(
        LocalSigner::from_pkcs12(key("v1"), b"garbage", PASSWORD),
        Err(Error::InvalidMaterial)
    ));
    assert!(matches!(
        LocalSigner::from_pem(key("v1"), b"garbage", PASSWORD, &m.certificate_pem),
        Err(Error::InvalidMaterial)
    ));
    assert!(matches!(
        LocalSigner::from_pem(
            key("v1"),
            &m.encrypted_private_key_pem,
            PASSWORD,
            &other.certificate_pem
        ),
        Err(Error::InvalidMaterial)
    ));
    let private =
        PKey::private_key_from_pem_passphrase(&m.encrypted_private_key_pem, PASSWORD.as_bytes())
            .unwrap();
    assert!(matches!(
        LocalSigner::from_pem(
            key("v1"),
            &private.private_key_to_pem_pkcs8().unwrap(),
            PASSWORD,
            &m.certificate_pem
        ),
        Err(Error::InvalidMaterial)
    ));
    let cert = X509::from_pem(&m.certificate_pem).unwrap();
    let mut pfx = openssl::pkcs12::Pkcs12::builder();
    pfx.cert(&cert);
    let no_private = pfx.build2(PASSWORD).unwrap().to_der().unwrap();
    assert!(matches!(
        LocalSigner::from_pkcs12(key("v1"), &no_private, PASSWORD),
        Err(Error::InvalidMaterial)
    ));
    let error = Error::InvalidMaterial;
    assert_eq!(
        format!("{error}: {error:?}"),
        "invalid signing material or password: InvalidMaterial"
    );
    assert!(std::error::Error::source(&error).is_none());
}

// Certificates are generated through OpenSSL, including deliberate policy failures.
fn certificate(private: &PKey<Private>, start: i64, end: i64, digital: Option<bool>) -> Vec<u8> {
    let mut name = openssl::x509::X509NameBuilder::new().unwrap();
    name.append_entry_by_nid(Nid::COMMONNAME, "Synthetic policy test")
        .unwrap();
    let name = name.build();
    let mut b = X509::builder().unwrap();
    b.set_version(2).unwrap();
    b.set_serial_number(&BigNum::from_u32(5).unwrap().to_asn1_integer().unwrap())
        .unwrap();
    b.set_subject_name(&name).unwrap();
    b.set_issuer_name(&name).unwrap();
    b.set_pubkey(private).unwrap();
    b.set_not_before(&Asn1Time::from_unix(start).unwrap())
        .unwrap();
    b.set_not_after(&Asn1Time::from_unix(end).unwrap()).unwrap();
    if let Some(digital) = digital {
        let mut ku = KeyUsage::new();
        if digital {
            ku.digital_signature();
        } else {
            ku.key_encipherment();
        }
        b.append_extension(ku.build().unwrap()).unwrap();
    }
    b.sign(private, MessageDigest::sha256()).unwrap();
    b.build().to_pem().unwrap()
}
fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
#[test]
fn certificate_dates_usage_curve_and_pin_policy() {
    let m = material();
    let private =
        PKey::private_key_from_pem_passphrase(&m.encrypted_private_key_pem, PASSWORD.as_bytes())
            .unwrap();
    let now = now();
    for pem in [
        certificate(&private, now - 200, now - 100, Some(true)),
        certificate(&private, now + 100, now + 200, Some(true)),
    ] {
        let public = PublicMaterial::from_certificate_pem(&pem).unwrap();
        assert_eq!(public.check_usable(), Err(Error::PolicyRejected));
        assert!(public.certificate_dates().is_ok()); // Expired history remains inspectable.
        assert!(matches!(
            LocalSigner::from_pem(key("v1"), &m.encrypted_private_key_pem, PASSWORD, &pem),
            Err(Error::PolicyRejected)
        ));
    }
    for usage in [None, Some(false)] {
        assert!(matches!(
            PublicMaterial::from_certificate_pem(&certificate(
                &private,
                now - 10,
                now + 100,
                usage
            )),
            Err(Error::PolicyRejected)
        ));
    }
    let p384 = PKey::from_ec_key(
        EcKey::generate(&EcGroup::from_curve_name(Nid::SECP384R1).unwrap()).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        PublicMaterial::from_certificate_pem(&certificate(&p384, now - 10, now + 100, Some(true))),
        Err(Error::UnsupportedAlgorithm)
    ));
    assert!(matches!(
        TrustedKey::certificate(key("v1"), &m.certificate_pem, [0; 32], KeyState::Active),
        Err(Error::PolicyRejected)
    ));
}

#[test]
fn rotation_retirement_revocation_and_multiple_logical_keys() {
    let one = material();
    let two = material();
    let other = material();
    let id_other = KeyRef {
        key: "invoices".into(),
        ..key("v1")
    };
    let signers = vec![
        signer(key("v1"), &one),
        signer(key("v2"), &two),
        signer(id_other.clone(), &other),
    ];
    let public = PublicKeySet::new(vec![
        (key("v1"), signers[0].public_material()),
        (key("v2"), signers[1].public_material()),
        (id_other.clone(), signers[2].public_material()),
    ])
    .unwrap();
    let policy = Policy::new(vec![
        trusted(key("v1"), &one, KeyState::Active),
        trusted(key("v2"), &two, KeyState::Retired),
        trusted(id_other.clone(), &other, KeyState::Active),
    ])
    .unwrap();
    let mut service = SigningService::new(LocalProvider::new(signers).unwrap(), public, policy);
    let old = block_on(service.sign(&key("v1"), b"message")).unwrap();
    assert_eq!(
        block_on(service.sign(&key("v2"), b"message")),
        Err(Error::PolicyRejected)
    );
    service
        .policy_mut()
        .set_state(&key("v2"), KeyState::Active)
        .unwrap();
    assert_eq!(
        block_on(service.sign(&key("v1"), b"message")),
        Err(Error::PolicyRejected)
    );
    block_on(service.verify(b"message", &old)).unwrap();
    block_on(service.sign(&id_other, b"independent key")).unwrap();
    let new = block_on(service.sign(&key("v2"), b"message")).unwrap();
    service
        .policy_mut()
        .set_state(&key("v1"), KeyState::Revoked)
        .unwrap();
    assert_eq!(
        block_on(service.verify(b"message", &old)),
        Err(Error::PolicyRejected)
    );
    assert_eq!(
        service.policy_mut().set_state(&key("v1"), KeyState::Active),
        Err(Error::PolicyRejected)
    );
    block_on(service.verify(b"message", &new)).unwrap();
    let duplicate = trusted(key("v3"), &one, KeyState::Retired);
    assert_eq!(
        service.policy_mut().insert(duplicate),
        Err(Error::PolicyRejected)
    );
}

#[test]
fn duplicate_key_rejected_even_with_different_certificate_and_revoked_identity() {
    let m = material();
    let private =
        PKey::private_key_from_pem_passphrase(&m.encrypted_private_key_pem, PASSWORD.as_bytes())
            .unwrap();
    let renewed = certificate(&private, now() - 5, now() + 86400, Some(true));
    let renewed_public = PublicMaterial::from_certificate_pem(&renewed).unwrap();
    assert_ne!(renewed_public.certificate_pin(), Some(m.certificate_pin));
    let second = TrustedKey::certificate(
        KeyRef {
            provider: "different-scope".into(),
            key: "other".into(),
            version: "v2".into(),
        },
        &renewed,
        renewed_public.certificate_pin().unwrap(),
        KeyState::Active,
    )
    .unwrap();
    assert!(matches!(
        Policy::new(vec![trusted(key("v1"), &m, KeyState::Revoked), second]),
        Err(Error::PolicyRejected)
    ));
    assert!(matches!(
        Policy::new(vec![
            trusted(key("v1"), &m, KeyState::Active),
            trusted(key("v1"), &m, KeyState::Active)
        ]),
        Err(Error::PolicyRejected)
    ));
}

#[test]
fn public_source_cannot_create_trust_or_substitute_identity() {
    struct PanicSource;
    impl PublicKeySource for PanicSource {
        async fn public_key(&self, _: &KeyRef) -> Result<ResolvedPublicKey> {
            panic!("untrusted envelope must not contact public source")
        }
    }
    let m = material();
    let envelope = block_on(service(&m).sign(&key("v1"), b"message")).unwrap();
    let verifier = Verifier::new(PanicSource, Policy::new(vec![]).unwrap());
    assert_eq!(
        block_on(verifier.verify(b"message", &envelope)),
        Err(Error::UnknownKey)
    );
    let other = material();
    let substituted = PublicKeySet::new(vec![(
        key("v1"),
        PublicMaterial::from_certificate_pem(&other.certificate_pem).unwrap(),
    )])
    .unwrap();
    let verifier = Verifier::new(
        substituted,
        Policy::new(vec![trusted(key("v1"), &m, KeyState::Active)]).unwrap(),
    );
    assert_eq!(
        block_on(verifier.verify(b"message", &envelope)),
        Err(Error::PolicyRejected)
    );
}

#[test]
fn backend_wrong_version_double_hash_and_failure_are_not_retried() {
    struct BadBackend {
        local: LocalSigner,
        mode: u8,
    }
    impl SigningBackend for BadBackend {
        async fn sign_digest(
            &self,
            key: &KeyRef,
            digest: Sha256Digest,
        ) -> Result<BackendSignature> {
            if self.mode == 2 {
                return Err(Error::BackendUnavailable);
            }
            let mut signed = self
                .local
                .sign_digest(
                    key,
                    if self.mode == 1 {
                        Sha256Digest::of(&digest.0)
                    } else {
                        digest
                    },
                )
                .await?;
            if self.mode == 0 {
                signed.key_ref.version = "unexpected".into();
            }
            Ok(signed)
        }
    }
    let m = material();
    for (mode, error) in [
        (0, Error::PolicyRejected),
        (1, Error::InvalidSignature),
        (2, Error::BackendUnavailable),
    ] {
        let local = signer(key("v1"), &m);
        let public = PublicKeySet::new(vec![(key("v1"), local.public_material())]).unwrap();
        let s = SigningService::new(
            BadBackend { local, mode },
            public,
            Policy::new(vec![trusted(key("v1"), &m, KeyState::Active)]).unwrap(),
        );
        assert_eq!(block_on(s.sign(&key("v1"), b"bytes")), Err(error));
    }
}

#[test]
fn certificate_free_provider_public_material_supported() {
    let m = material();
    let local = signer(key("v1"), &m);
    let public =
        PublicMaterial::from_public_key_der(&local.public_material().public_key_der().unwrap())
            .unwrap();
    assert_eq!(public.certificate_pin(), None);
    let source = PublicKeySet::new(vec![(key("v1"), public.clone())]).unwrap();
    let resolved = block_on(source.clone().public_key(&key("v1"))).unwrap();
    assert_eq!(resolved.key_ref, key("v1"));
    assert_eq!(resolved.material.certificate_pin(), None);
    assert!(resolved.material.same_public_key(&public));
    let policy = Policy::new(vec![
        TrustedKey::public_key(key("v1"), public, KeyState::Active).unwrap(),
    ])
    .unwrap();
    let s = SigningService::new(local, source, policy);
    let envelope = block_on(s.sign(&key("v1"), b"provider-neutral")).unwrap();
    block_on(s.verify(b"provider-neutral", &envelope)).unwrap();
}

#[test]
fn macless_pfx_rejected_even_when_key_is_encrypted() {
    use foreign_types::ForeignType;
    use std::ffi::CString;
    let m = material();
    let private =
        PKey::private_key_from_pem_passphrase(&m.encrypted_private_key_pem, PASSWORD.as_bytes())
            .unwrap();
    let cert = X509::from_pem(&m.certificate_pem).unwrap();
    let password = CString::new(PASSWORD).unwrap();
    // OpenSSL documents mac_iter=-1 as disabling the archive MAC. The binding's
    // builder unconditionally adds a MAC, so use its maintained C API for this
    // intentionally invalid fixture. Returned ownership passes to Pkcs12.
    let raw = unsafe {
        openssl_sys::PKCS12_create(
            password.as_ptr(),
            std::ptr::null(),
            private.as_ptr(),
            cert.as_ptr(),
            std::ptr::null_mut(),
            0,
            0,
            2048,
            -1,
            0,
        )
    };
    assert!(!raw.is_null());
    let archive = unsafe { openssl::pkcs12::Pkcs12::from_ptr(raw) };
    let der = archive.to_der().unwrap();
    assert!(archive.parse2(PASSWORD).is_ok());
    assert!(matches!(
        LocalSigner::from_pkcs12(key("v1"), &der, PASSWORD),
        Err(Error::InvalidMaterial)
    ));
}
