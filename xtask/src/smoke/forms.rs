//! W3C forms keyboard/accessibility observations, preserving sanitized evidence.
use super::{
    Server,
    browser::{Browser, Client},
    http, require,
};
use crate::tooling::{self, Result};
use serde_json::{Value, json};
use std::path::Path;
const TAB: &str = "\u{e004}";
fn focus(c: &mut Client, selector: &str) -> Result<()> {
    c.until(
        &format!(
            "document.activeElement===document.querySelector({})",
            json!(selector)
        ),
        "keyboard focus",
    )
}
fn english(c: &mut Client) -> Result<()> {
    c.equal("document.documentElement.lang", json!("en"))?;
    c.equal("document.querySelectorAll('script').length", json!(0))
}
fn labels(c: &mut Client) -> Result<()> {
    for (field, label) in [("name", "Your name"), ("email", "Email address")] {
        c.equal(
            &format!("document.querySelector('label[for={field}]').textContent"),
            json!(label),
        )?;
        c.check(&format!("document.querySelector('label[for={field}]').control===document.getElementById('{field}')"))?;
    }
    Ok(())
}
fn stage(report: &mut Value, name: &str) {
    report["stage"] = json!(name);
}
fn passed(report: &mut Value, name: &str) {
    report["checks"].as_array_mut().unwrap().push(json!(name));
}
pub fn run(root: &Path, out: &Path, profile: &str) -> Result<()> {
    let mut report = json!({"task_id":"YT-065","status":"fail","profile":profile,"stage":"prerequisites","checks":[],"browser_version":null,"cleanup":"pending","process_exit_codes":{},"not_verified":["real_os_ime","screen_reader","webkit","mobile","zoom","accessibility_conformance","sla","production_qualification"]});
    let mut server = None;
    let mut browser = None;
    let checks = (|| {
        stage(&mut report, "server_startup");
        server = Some(Server::start(
            root,
            out,
            "yetlet-forms-validation",
            profile,
            "YETLET_FORMS_LISTEN",
            &[],
            "/form",
        )?);
        stage(&mut report, "driver_startup");
        browser = Some(Browser::start(root, out, true)?);
        report["browser_version"] = json!(browser.as_ref().unwrap().version);
        exercise(
            browser.as_mut().unwrap().client()?,
            &server.as_ref().unwrap().base,
            &mut report,
        )
    })();
    finalize(
        out,
        &mut report,
        checks,
        |report| {
            let result = browser.as_mut().map_or(Ok(()), Browser::stop);
            if let Some(b) = &browser {
                report["process_exit_codes"]["geckodriver"] = json!(b.exit_code());
            }
            result
        },
        |report| {
            let result = server.as_mut().map_or(Ok(()), |s| s.stop(false));
            if let Some(s) = &server
                && let Ok(bytes) = std::fs::read(s.log.with_extension("exit.json"))
                && let Ok(v) = serde_json::from_slice::<Value>(&bytes)
            {
                report["process_exit_codes"]["forms_server"] = v["native_exit_code"].clone();
            }
            result
        },
    )
}
fn remove_log(entry: std::io::Result<std::fs::DirEntry>) -> Result<()> {
    let remove = || -> std::io::Result<()> {
        let path = entry?.path();
        if path.extension().is_some_and(|e| e == "log") {
            std::fs::remove_file(path)?;
        }
        Ok(())
    };
    remove().map_err(|_| tooling::Error {
        code: 125,
        message: "forms log cleanup failed".into(),
    })
}
fn finalize(
    out: &Path,
    report: &mut Value,
    checks: Result<()>,
    stop_browser: impl FnOnce(&mut Value) -> Result<()>,
    stop_server: impl FnOnce(&mut Value) -> Result<()>,
) -> Result<()> {
    // Keep attempting cleanup and the required sanitized report after any failure.
    let browser = stop_browser(report);
    let server = stop_server(report);
    let mut first_error = None;
    let mut cleanup_errors = Vec::new();
    let mut record = |label, result: Result<()>| {
        if let Err(error) = result {
            first_error.get_or_insert(error);
            cleanup_errors.push(label);
        }
    };
    record("BrowserCleanup", browser);
    record("ServerCleanup", server);
    // The original forms report explicitly excludes raw process/driver output.
    match std::fs::read_dir(out) {
        Ok(entries) => {
            for entry in entries {
                record("LogCleanup", remove_log(entry));
            }
        }
        Err(_) => record(
            "LogDirectoryRead",
            Err(tooling::Error {
                code: 125,
                message: "forms log directory read failed".into(),
            }),
        ),
    }
    report["cleanup"] = json!(if cleanup_errors.is_empty() {
        "pass"
    } else {
        "fail"
    });
    report["cleanup_errors"] = json!(cleanup_errors);
    report["failure_type"] = if checks.is_err() {
        json!("ToolFailure")
    } else {
        Value::Null
    };
    report["status"] = json!("fail");
    if checks.is_ok() && cleanup_errors.is_empty() {
        report["status"] = json!("pass");
        stage(report, "complete");
    }
    let receipt = std::fs::write(
        out.join("forms-accessibility-firefox.json"),
        tooling::json_bytes(report),
    )
    .map_err(|_| tooling::Error {
        code: 125,
        message: "forms sanitized report write failed".into(),
    });
    if receipt.is_err() {
        report["status"] = json!("fail");
    }
    if let Some(error) = first_error {
        return Err(error);
    }
    receipt?;
    checks
}
fn exercise(c: &mut Client, base: &str, r: &mut Value) -> Result<()> {
    stage(r, "initial_document");
    c.navigate(&format!("{base}/form"))?;
    english(c)?;
    labels(c)?;
    c.check("document.querySelector('form').noValidate===true")?;
    passed(r, "initial_language_labels_no_script");
    stage(r, "tab_order");
    c.firefox()?.keys(TAB)?;
    focus(c, "#name")?;
    c.firefox()?.keys(&format!("x{TAB}"))?;
    focus(c, "#email")?;
    c.firefox()?.keys(&format!("invalid{TAB}"))?;
    focus(c, "button[type=submit]")?;
    passed(r, "tab_name_email_submit");
    stage(r, "invalid_post");
    c.firefox()?.keys("\u{e007}")?;
    c.until("location.href.endsWith('/form/submit')", "invalid POST")?;
    focus(c, "#name")?;
    english(c)?;
    labels(c)?;
    for (field, message) in [
        ("name", "Enter a name between 2 and 40 characters."),
        ("email", "Enter a valid example email address."),
    ] {
        c.equal(
            &format!("document.getElementById('{field}').getAttribute('aria-invalid')"),
            json!("true"),
        )?;
        c.equal(
            &format!("document.getElementById('{field}').getAttribute('aria-describedby')"),
            json!(format!("{field}-error")),
        )?;
        c.equal(
            &format!("document.getElementById('{field}-error').textContent"),
            json!(message),
        )?;
        c.equal(
            &format!("document.getElementById('{field}-error').getAttribute('role')"),
            json!("alert"),
        )?;
    }
    c.equal("document.querySelectorAll('[autofocus]').length", json!(1))?;
    passed(r, "invalid_first_error_focus_language_labels_descriptions");
    stage(r, "unicode_and_email_focus");
    let name = "Zoé 漢字";
    c.firefox()?.replace_text(name)?;
    c.equal("document.getElementById('name').value", json!(name))?;
    c.firefox()?.keys(&format!("{TAB}{TAB}\u{e007}"))?;
    focus(c, "#email")?;
    english(c)?;
    c.equal("document.getElementById('name').value", json!(name))?;
    c.equal(
        "document.getElementById('name').hasAttribute('aria-invalid')",
        json!(false),
    )?;
    c.equal("document.querySelectorAll('[autofocus]').length", json!(1))?;
    passed(r, "unicode_key_input_preserved_email_first_error_focus");
    stage(r, "success_prg");
    c.firefox()?.replace_text("unicode@example.test")?;
    c.firefox()?.keys(&format!("{TAB}\u{e007}"))?;
    c.until(
        &format!("location.href==={}", json!(format!("{base}/form/success"))),
        "success GET",
    )?;
    english(c)?;
    c.equal(
        "document.querySelector('h1').textContent",
        json!("Form accepted"),
    )?;
    c.equal(
        "document.querySelector('main p').textContent",
        json!("No data was saved."),
    )?;
    c.check(&format!(
        "!document.body.textContent.includes({})",
        json!(name)
    ))?;
    require(
        c.firefox()?.command("/cookie", "GET", None)? == json!([]),
        "no forms cookie",
    )?;
    c.equal("localStorage.length+sessionStorage.length", json!(0))?;
    c.firefox()?.command("/refresh", "POST", Some(&json!({})))?;
    require(
        c.firefox()?.command("/url", "GET", None)? == format!("{base}/form/success"),
        "refresh remains success GET",
    )?;
    english(c)?;
    let response = http::form(
        base,
        "/form/submit",
        "",
        &[("name", name), ("email", "unicode@example.test")],
    )?;
    require(
        response.status == 303
            && response.header("location")? == "/form/success"
            && response.header("cache-control")? == "no-store"
            && !response.headers.contains_key("set-cookie"),
        "separate raw HTTP stateless 303 PRG",
    )?;
    c.navigate(&format!("{base}/form"))?;
    c.equal("document.getElementById('name').value", json!(""))?;
    c.equal("document.getElementById('email').value", json!(""))?;
    passed(r, "success_language_no_state_303_refresh_empty_form");
    Ok(())
}

#[cfg(test)]
mod parent_suite_cleanup {
    use super::*;
    use std::{cell::RefCell, fs};

    fn report() -> Value {
        json!({"task_id":"YT-065","status":"fail","profile":"release","stage":"success_prg","checks":["completed check"],"browser_version":"fixture","cleanup":"pending","process_exit_codes":{},"not_verified":["real_os_ime"]})
    }

    #[test]
    fn browser_and_server_errors_retain_125_and_first_error_after_all_cleanup() {
        for browser_fails in [true, false] {
            let out = tempfile::tempdir().unwrap();
            fs::write(out.path().join("driver.log"), "private raw output").unwrap();
            let native_receipt = b"{\"native_exit_code\":23,\"signal\":null}";
            fs::write(out.path().join("driver.exit.json"), native_receipt).unwrap();
            let mut r = report();
            let attempts = RefCell::new(Vec::new());
            let error = finalize(
                out.path(),
                &mut r,
                Err(tooling::fail("private primary diagnostic")),
                |r| {
                    attempts.borrow_mut().push("browser");
                    r["process_exit_codes"]["geckodriver"] = json!(23);
                    if browser_fails {
                        Err(tooling::Error {
                            code: 125,
                            message: "browser cleanup failed".into(),
                        })
                    } else {
                        Ok(())
                    }
                },
                |r| {
                    attempts.borrow_mut().push("server");
                    r["process_exit_codes"]["forms_server"] = Value::Null;
                    Err(tooling::Error {
                        code: 125,
                        message: "server cleanup failed".into(),
                    })
                },
            )
            .unwrap_err();
            assert_eq!(error.code, 125);
            assert_eq!(
                error.message,
                if browser_fails {
                    "browser cleanup failed"
                } else {
                    "server cleanup failed"
                }
            );
            assert_eq!(*attempts.borrow(), ["browser", "server"]);
            assert!(!out.path().join("driver.log").exists());
            assert_eq!(
                fs::read(out.path().join("driver.exit.json")).unwrap(),
                native_receipt
            );
            let bytes = fs::read(out.path().join("forms-accessibility-firefox.json")).unwrap();
            assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap(), r);
            assert_eq!(r["status"], "fail");
            assert_eq!(r["cleanup"], "fail");
            assert_eq!(r["failure_type"], "ToolFailure");
            assert_eq!(r["stage"], "success_prg");
            assert_eq!(r["checks"], json!(["completed check"]));
            assert_eq!(r["not_verified"], json!(["real_os_ime"]));
            assert_eq!(
                r["process_exit_codes"],
                json!({"geckodriver":23,"forms_server":null})
            );
            assert_eq!(
                r["cleanup_errors"],
                if browser_fails {
                    json!(["BrowserCleanup", "ServerCleanup"])
                } else {
                    json!(["ServerCleanup"])
                }
            );
            assert!(!String::from_utf8(bytes).unwrap().contains("private"));
        }
    }

    #[test]
    fn log_removal_error_still_removes_other_logs_and_writes_sanitized_failure() {
        let out = tempfile::tempdir().unwrap();
        fs::create_dir(out.path().join("blocked.log")).unwrap();
        fs::write(out.path().join("other.log"), "private raw output").unwrap();
        let mut r = report();
        let error = finalize(out.path(), &mut r, Ok(()), |_| Ok(()), |_| Ok(())).unwrap_err();
        assert_eq!(error.code, 125);
        assert_eq!(error.message, "forms log cleanup failed");
        assert!(!out.path().join("other.log").exists());
        assert_eq!(r["status"], "fail");
        assert_eq!(r["cleanup"], "fail");
        assert_eq!(r["cleanup_errors"], json!(["LogCleanup"]));
        assert!(r["failure_type"].is_null());
        let bytes = fs::read(out.path().join("forms-accessibility-firefox.json")).unwrap();
        assert_eq!(serde_json::from_slice::<Value>(&bytes).unwrap(), r);
        assert!(!String::from_utf8(bytes).unwrap().contains("blocked.log"));
    }

    #[test]
    fn directory_read_error_retains_125_and_first_failure_despite_failed_receipt() {
        let fixture = tempfile::tempdir().unwrap();
        let out = fixture.path().join("private-not-a-directory");
        fs::write(&out, "fixture").unwrap();
        let mut r = report();
        let attempts = RefCell::new(Vec::new());
        let error = finalize(
            &out,
            &mut r,
            Ok(()),
            |_| {
                attempts.borrow_mut().push("browser");
                Ok(())
            },
            |_| {
                attempts.borrow_mut().push("server");
                Ok(())
            },
        )
        .unwrap_err();
        assert_eq!(error.code, 125);
        assert_eq!(error.message, "forms log directory read failed");
        assert_eq!(*attempts.borrow(), ["browser", "server"]);
        assert_eq!(r["cleanup_errors"], json!(["LogDirectoryRead"]));
        assert_eq!(r["status"], "fail");
    }

    #[test]
    fn directory_entry_error_is_infrastructure_without_raw_io_details() {
        let error = remove_log(Err(std::io::Error::other(
            "private path and raw diagnostic",
        )))
        .unwrap_err();
        assert_eq!(error.code, 125);
        assert_eq!(error.message, "forms log cleanup failed");
    }

    #[test]
    fn required_report_failure_cannot_pass_or_mask_earlier_cleanup_error() {
        for browser_fails in [false, true] {
            let out = tempfile::tempdir().unwrap();
            fs::create_dir(out.path().join("forms-accessibility-firefox.json")).unwrap();
            fs::write(out.path().join("raw.log"), "private raw output").unwrap();
            let mut r = report();
            let error = finalize(
                out.path(),
                &mut r,
                Ok(()),
                |_| {
                    if browser_fails {
                        Err(tooling::Error {
                            code: 125,
                            message: "first browser cleanup failed".into(),
                        })
                    } else {
                        Ok(())
                    }
                },
                |_| Ok(()),
            )
            .unwrap_err();
            assert_eq!(error.code, 125);
            assert_eq!(
                error.message,
                if browser_fails {
                    "first browser cleanup failed"
                } else {
                    "forms sanitized report write failed"
                }
            );
            assert_eq!(r["status"], "fail");
            assert!(!out.path().join("raw.log").exists());
        }
    }

    #[test]
    fn successful_cleanup_preserves_primary_status_and_sanitized_fields() {
        for code in [1, 23, 124, 125, 130] {
            let out = tempfile::tempdir().unwrap();
            let mut r = report();
            let error = finalize(
                out.path(),
                &mut r,
                Err(tooling::Error {
                    code,
                    message: "private primary diagnostic".into(),
                }),
                |_| Ok(()),
                |_| Ok(()),
            )
            .unwrap_err();
            assert_eq!(error.code, code);
            assert_eq!(error.message, "private primary diagnostic");
            assert_eq!(r["cleanup"], "pass");
            assert_eq!(r["cleanup_errors"], json!([]));
            assert_eq!(r["status"], "fail");
            assert_eq!(r["failure_type"], "ToolFailure");
            assert_eq!(r["stage"], "success_prg");
            assert!(!r.to_string().contains("private"));
        }
        let out = tempfile::tempdir().unwrap();
        let mut r = report();
        finalize(out.path(), &mut r, Ok(()), |_| Ok(()), |_| Ok(())).unwrap();
        assert_eq!(r["status"], "pass");
        assert_eq!(r["stage"], "complete");
        assert_eq!(r["cleanup"], "pass");
        assert!(r["failure_type"].is_null());
    }
}
