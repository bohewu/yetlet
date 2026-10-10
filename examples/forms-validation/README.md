# Forms Validation — Stateless Native SSR

This self-contained application demonstrates server-rendered HTML forms and ordinary Axum `Form<T>` extraction. It has no JavaScript, WASM, database, Session, authentication, server-side storage, generic forms framework or Yetlet ORM.

## Run

From the repository root:

```bash
cargo run -p yetlet-forms-validation
```

The default listener is `127.0.0.1:3005`, overridable for local testing through `YETLET_FORMS_LISTEN`. Routes:

- `/form`: native GET SSR form, semantic label/input associations and strict CSP.
- `/form/submit`: POST form input, 2 KiB body limit, strict field decoding and server-side field validation; native extractor errors such as unknown fields and application validation errors return 422 with escaped values, associated error descriptions and `autofocus` on the first error.
- `/form/success`: GET generic success page after a valid POST (HTTP 303 redirect). This is a deliberately stateless **POST/Redirect/GET** demonstration: no write occurs and the success page does not echo submitted data.

The `name` limit is 2–40 characters; email validation is deliberately **illustrative**, not a deliverable email-address validator. HTML attribute values and text are escaped. Responses use `Cache-Control: no-store` and `nosniff`. Forms include no CSRF token only because **there is no authentication, persistence, transaction or state-changing side effect**. A credentialed or mutating form requires its own explicitly selected Session/Auth/CSRF policy and separate security review. `SameSite` or lack of a Session does not prove CSRF protection.

The sample owns English at the root document through `DocumentMetadata` and `LanguageTag`; it does not add a nested `lang` workaround or an arbitrary head manager. The default Yetlet document language remains unchanged for applications that keep using `Page::new`.

## Verify

```bash
cargo test --locked -p yetlet-forms-validation --all-targets
cargo build --locked -p yetlet-forms-validation
python3 scripts/smoke_forms_validation.py
cargo tree --locked -p yetlet-forms-validation
python3 scripts/check_samples.py
```

This is a bounded prototype for ergonomics and accessible markup, not a production identity, registration or email-verification flow. See [M9c verification](../../docs/M9C-VERIFICATION.md).
