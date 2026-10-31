//! Fixed public SDK closure bundled inside the independently installable CLI.
use std::{collections::BTreeMap, sync::LazyLock};

pub(crate) static FILES: LazyLock<BTreeMap<String, String>> = LazyLock::new(|| {
    serde_json::from_str(include_str!("../resources/framework.json"))
        .expect("compiled public SDK source closure")
});
