// The public CLI namespace shares the backend loader without relaxing unknown fields.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cli {
    #[serde(default)]
    pub rust: Rust,
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
