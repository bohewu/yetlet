//! Bounded source metrics and original local measurements; no performance gate is inferred.
pub mod baseline;
pub mod dx;
pub mod dynamic;
pub mod render;
use crate::{
    smoke::require,
    tooling::{Result, fail},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs,
    io::Write,
    path::{Path, PathBuf},
};
#[derive(Debug, PartialEq, Eq)]
pub struct Options {
    pub values: BTreeMap<String, OsString>,
}
impl Options {
    pub fn parse(suite: &str, args: &[OsString]) -> std::result::Result<Self, &'static str> {
        let allowed: &[&str] = match suite {
            "baseline" => &["--profile", "--samples", "--browser-runs", "--out"],
            "render-strategies" => &[
                "--profile",
                "--rows",
                "--updates",
                "--rounds",
                "--seed",
                "--capture-production-before",
                "--production-before",
                "--out",
            ],
            "sample-dx" => &["--out"],
            "scene-dynamic" => &[],
            _ => return Err("unknown measurement"),
        };
        let mut values = BTreeMap::new();
        let (chunks, remainder) = args.as_chunks::<2>();
        for pair in chunks {
            let k = pair[0].to_str().ok_or("option encoding")?;
            if !allowed.contains(&k) || values.insert(k.into(), pair[1].clone()).is_some() {
                return Err("unknown or repeated measurement option");
            }
        }
        if !remainder.is_empty() {
            return Err("measurement option needs value");
        }
        let options = Self { values };
        let p = options.string("--profile", "release");
        if p != "debug" && p != "release" {
            return Err("profile must be debug|release");
        }
        for (name, default, lo, hi) in match suite {
            "baseline" => vec![("--samples", 30, 5, 500), ("--browser-runs", 5, 1, 30)],
            "render-strategies" => vec![
                ("--rows", 200, 3, 1000),
                ("--updates", 500, 1, 10000),
                ("--rounds", 10, 1, 30),
            ],
            _ => vec![],
        } {
            let n = options
                .number(name, default)
                .map_err(|_| "invalid integer option")?;
            if n < lo || n > hi {
                return Err("measurement bounds violated");
            }
        }
        if suite == "render-strategies" {
            options.seed_literal().map_err(|_| "invalid integer seed")?;
        }
        Ok(options)
    }
    pub fn string(&self, key: &str, default: &str) -> String {
        self.values
            .get(key)
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| default.into())
    }
    pub fn number(&self, key: &str, default: usize) -> Result<usize> {
        match self.values.get(key) {
            None => Ok(default),
            Some(v) => v
                .to_str()
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| fail("invalid integer")),
        }
    }
    /// Keep Python int's signed, arbitrary-width decimal value as a JS literal.
    /// Conversion to Number and then the benchmark's u32 seed happens in-browser.
    pub fn seed_literal(&self) -> Result<String> {
        let raw = self.string("--seed", "7");
        let raw = raw.trim();
        let (negative, digits) = match raw.as_bytes().first() {
            Some(b'-') => (true, &raw[1..]),
            Some(b'+') => (false, &raw[1..]),
            _ => (false, raw),
        };
        let nd = regex::Regex::new(r"^\p{Nd}$").unwrap();
        let mut decimal = String::new();
        for group in digits.split('_') {
            require(!group.is_empty(), "invalid integer seed")?;
            for c in group.chars() {
                if c.is_ascii_digit() {
                    decimal.push(c);
                } else {
                    require(nd.is_match(&c.to_string()), "invalid integer seed")?;
                    // Unicode decimal digits occur in contiguous groups of ten
                    // (some mathematical styles adjoin several such groups).
                    let mut zero = c as u32;
                    while char::from_u32(zero - 1).is_some_and(|p| nd.is_match(&p.to_string())) {
                        zero -= 1;
                    }
                    decimal.push(char::from(b'0' + ((c as u32 - zero) % 10) as u8));
                }
            }
        }
        let decimal = decimal.trim_start_matches('0');
        Ok(if decimal.is_empty() {
            "0".into()
        } else {
            format!("{}{decimal}", if negative { "-" } else { "" })
        })
    }
    pub fn path(&self, key: &str, default: impl AsRef<Path>) -> PathBuf {
        self.values
            .get(key)
            .map(PathBuf::from)
            .unwrap_or_else(|| default.as_ref().into())
    }
}
pub fn run(root: &Path, suite: &str, args: &[OsString]) -> Result<()> {
    let o = Options::parse(suite, args).map_err(|e| crate::tooling::Error {
        code: 2,
        message: e.into(),
    })?;
    match suite {
        "scene-dynamic" => dynamic::run(root),
        "sample-dx" => dx::run(root, &o),
        "baseline" => baseline::run(root, &o),
        "render-strategies" => render::run(root, &o),
        _ => Err(fail("unknown measurement")),
    }
}
pub fn output(path: &Path, value: &Value) -> Result<()> {
    if let Some(p) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(p)?
    }
    fs::write(path, crate::tooling::json_bytes(value))?;
    Ok(())
}
pub fn distribution(values: &[f64]) -> Result<Value> {
    require(
        !values.is_empty() && values.iter().all(|v| v.is_finite() && *v >= 0.),
        "invalid raw measurement",
    )?;
    let mut v = values.to_vec();
    v.sort_by(f64::total_cmp);
    let n = v.len();
    let round = |x: f64| (x * 1000.).round_ties_even() / 1000.;
    Ok(
        json!({"samples":n,"median_ms":round(if n.is_multiple_of(2){(v[n/2-1]+v[n/2])/2.}else{v[n/2]}),"p95_ms":round(v[(n*95).div_ceil(100)-1]),"min_ms":round(v[0]),"max_ms":round(v[n-1]),"raw_ms":values}),
    )
}
pub fn gzip(bytes: &[u8]) -> Result<usize> {
    let mut w = flate2::GzBuilder::new()
        .mtime(0)
        .operating_system(3)
        .write(Vec::new(), flate2::Compression::best());
    w.write_all(bytes)?;
    Ok(w.finish()?.len())
}
pub fn file_metrics(path: &Path) -> Result<Value> {
    let b = fs::read(path)?;
    Ok(json!({"bytes":b.len(),"gzip_bytes":gzip(&b)?,"sha256":crate::tooling::sha(&b)}))
}
pub fn environment(root: &Path) -> Result<Value> {
    let rustc = crate::tooling::execute(
        root,
        &crate::tooling::tool("rustc", ".cargo/bin")?,
        &["--version".into()],
        true,
    )?;
    Ok(
        json!({"os":std::env::consts::OS,"architecture":std::env::consts::ARCH,"rustc":String::from_utf8_lossy(&rustc).trim(),"tool":"native unpublished Rust xtask"}),
    )
}
