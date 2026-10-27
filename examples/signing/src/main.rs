//! Disposable signing workflow; the key CLI manages separate persistent identities.
use std::process::ExitCode;
use yetlet_signing::{
    Error, KeyRef, KeyState, LocalProvider, LocalSigner, Policy, PublicKeySet, SigningService,
    TrustedKey, Verifier, generate_self_signed,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    let Ok(password) = std::env::var("YETLET_SIGNING_PASSWORD") else {
        eprintln!("Set YETLET_SIGNING_PASSWORD to a nonempty disposable development password.");
        return ExitCode::FAILURE;
    };
    match demonstrate(&password).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Signing example failed: {error}");
            ExitCode::FAILURE
        }
    }
}

fn key(name: &str, version: &str) -> KeyRef {
    KeyRef {
        provider: "local_dev".into(),
        key: name.into(),
        version: version.into(),
    }
}

fn require(condition: bool) -> Result<(), Error> {
    condition.then_some(()).ok_or(Error::PolicyRejected)
}

async fn demonstrate(password: &str) -> Result<(), Error> {
    let first = generate_self_signed("documents-v1-disposable", password, 30)?;
    let second = generate_self_signed("documents-v2-disposable", password, 30)?;
    let invoices = generate_self_signed("invoices-v1-disposable", password, 30)?;
    let first_ref = key("documents", "v1");
    let second_ref = key("documents", "v2");
    let invoice_ref = key("invoices", "v1");

    let first_signer = LocalSigner::from_pem(
        first_ref.clone(),
        &first.encrypted_private_key_pem,
        password,
        &first.certificate_pem,
    )?;
    let second_signer = LocalSigner::from_pkcs12(second_ref.clone(), &second.pkcs12_der, password)?;
    let invoice_signer = LocalSigner::from_pem(
        invoice_ref.clone(),
        &invoices.encrypted_private_key_pem,
        password,
        &invoices.certificate_pem,
    )?;
    let public = PublicKeySet::new(vec![
        (first_ref.clone(), first_signer.public_material()),
        (second_ref.clone(), second_signer.public_material()),
        (invoice_ref.clone(), invoice_signer.public_material()),
    ])?;
    let make_policy = |first_state| {
        Policy::new(vec![
            TrustedKey::certificate(
                first_ref.clone(),
                &first.certificate_pem,
                first.certificate_pin,
                first_state,
            )?,
            TrustedKey::certificate(
                second_ref.clone(),
                &second.certificate_pem,
                second.certificate_pin,
                if first_state == KeyState::Active {
                    KeyState::Retired
                } else {
                    KeyState::Active
                },
            )?,
            TrustedKey::certificate(
                invoice_ref.clone(),
                &invoices.certificate_pem,
                invoices.certificate_pin,
                KeyState::Active,
            )?,
        ])
    };
    let backend = LocalProvider::new(vec![first_signer, second_signer, invoice_signer])?;
    let mut service = SigningService::new(backend, public.clone(), make_policy(KeyState::Active)?);
    let document = b"Yetlet signing reference: exact original bytes.";
    let original = service.sign(&first_ref, document).await?;
    service.verify(document, &original).await?;
    println!("PASS documents/v1: encrypted PEM sign and verification");

    require(
        service
            .verify(b"changed document", &original)
            .await
            .is_err(),
    )?;
    println!("PASS tampering: changed document rejected");

    service
        .policy_mut()
        .set_state(&second_ref, KeyState::Active)?;
    let rotated = service.sign(&second_ref, document).await?;
    service.verify(document, &original).await?;
    service.verify(document, &rotated).await?;
    require(service.sign(&first_ref, document).await.is_err())?;
    println!("PASS documents/v2: PFX sign; retired v1 still verifies and cannot sign");

    let invoice = service.sign(&invoice_ref, b"invoice bytes").await?;
    service.verify(b"invoice bytes", &invoice).await?;
    println!("PASS multiple keys: invoices/v1 remains usable after documents rotation");

    let mut verifier = Verifier::new(public, make_policy(KeyState::Retired)?);
    verifier.verify(document, &original).await?;
    verifier.verify(document, &rotated).await?;
    println!("PASS public-only verifier: no signing backend or private material");

    verifier
        .policy_mut()
        .set_state(&first_ref, KeyState::Revoked)?;
    require(verifier.verify(document, &original).await.is_err())?;
    verifier.verify(document, &rotated).await?;
    println!("PASS revocation: v1 rejected while v2 remains trusted");
    println!("Disposable in-memory demonstration complete; no key files were written.");
    Ok(())
}
