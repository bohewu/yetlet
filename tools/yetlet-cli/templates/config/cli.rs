// The public CLI namespace shares the backend loader without relaxing unknown fields.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cli {
    #[serde(default)]
    pub rust: Rust,
    #[serde(default)]
    pub styles: Option<Styles>,
    pub vite: Vite,
    pub generate: Generate,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rust {
    pub watch: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Vite {
    pub enabled: bool,
    pub root: String,
    pub auto_start: bool,
    pub auto_build: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Generate {
    pub k8s: K8s,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct K8s {
    pub replicas: u8,
}

impl Cli {
    fn validate(&self) -> io::Result<()> {
        let _ = (
            self.rust.watch,
            self.vite.enabled,
            self.vite.auto_start,
            self.vite.auto_build,
        );
        if self.styles.as_ref().is_some_and(|styles| !styles.valid(&self.vite)) { return Err(invalid()); }
        if !(1..=100).contains(&self.generate.k8s.replicas)
            || self.vite.root.is_empty()
            || self.vite.root.len() > 240
            || self.vite.root.contains('\\')
            || self.vite.root.contains(':')
            || self.vite.root.starts_with('/')
            || self
                .vite
                .root
                .trim_end_matches('/')
                .split('/')
                .any(|p| matches!(p, "" | "." | ".."))
        {
            return Err(invalid());
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Styles { pub enabled: bool, pub manifest: String, pub bin: String, pub output: String }
impl Styles {
    fn valid(&self, vite: &Vite) -> bool {
        let _ = self.enabled;
        self.manifest == "stylegen/Cargo.toml" && !self.bin.is_empty() && self.bin.len() <= 64
            && self.bin.as_bytes()[0].is_ascii_alphabetic()
            && self.bin.bytes().all(|b|b.is_ascii_alphanumeric() || matches!(b,b'-'|b'_'))
            && !["stylegen", ".yetlet", "src"].iter().any(|private| {
                let frontend = std::path::Path::new(vite.root.trim_end_matches('/'));
                frontend.starts_with(private) || std::path::Path::new(private).starts_with(frontend)
            })
            && if vite.enabled { self.output == format!("{}/style-output/app.generated.css",vite.root.trim_end_matches('/')) } else { self.output == "assets/generated/app.generated.css" }
    }
}
