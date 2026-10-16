//! Finite application-owned asset manifest. No filesystem paths or arbitrary head HTML.
use crate::asset_data::{ASSETS, EmbeddedAsset};
use serde::Deserialize;

pub const PREFIX: &str = "/form-proof-assets/";
const MAX_MANIFEST_BYTES: usize = 4096;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub schema_version: u32,
    pub stylesheets: Vec<String>,
    pub enhancement: String,
}

pub fn find(path: &str) -> Option<&'static EmbeddedAsset> {
    let name = path.strip_prefix(PREFIX)?;
    ASSETS.iter().find(|asset| asset.name == name)
}

impl Manifest {
    pub fn parse(input: &str) -> Result<Self, &'static str> {
        if input.len() > MAX_MANIFEST_BYTES {
            return Err("manifest exceeds limit");
        }
        let mut value: Self = serde_json::from_str(input).map_err(|_| "invalid manifest")?;
        if value.schema_version != 1 || value.stylesheets.is_empty() || value.stylesheets.len() > 8
        {
            return Err("unsupported manifest shape");
        }
        for path in &value.stylesheets {
            if find(path).is_none_or(|asset| asset.content_type != "text/css; charset=utf-8") {
                return Err("unknown stylesheet");
            }
        }
        if find(&value.enhancement)
            .is_none_or(|asset| asset.content_type != "text/javascript; charset=utf-8")
        {
            return Err("unknown enhancement");
        }
        let mut seen = std::collections::HashSet::new();
        value.stylesheets.retain(|path| seen.insert(path.clone()));
        Ok(value)
    }
}
