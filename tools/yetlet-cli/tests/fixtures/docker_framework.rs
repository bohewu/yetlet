//! Self-contained package/closure fixture; no Cargo invocation or registry access.
use std::{fs, path::Path};
pub fn create(root: &Path) {
    fs::create_dir_all(root).unwrap();
    fs::write(root.join("Cargo.toml"), "[workspace]\nresolver='3'\nmembers=['crates/yetlet-core','crates/yetlet-html','crates/yetlet-server-axum']\n[workspace.package]\nversion='0.1.0'\nedition='2024'\nrust-version='1.99'\nlicense='MIT OR Apache-2.0'\nrepository='https://github.com/bohewu/yetlet'\n[workspace.dependencies]\naxum='0.8'\ntokio={version='1',features=['macros','net','rt-multi-thread']}\n").unwrap();
    for (name, sources, dependencies) in [
        (
            "yetlet-core",
            vec!["binding.rs", "lib.rs"],
            "[dependencies]\n",
        ),
        (
            "yetlet-html",
            vec!["lib.rs"],
            "[dependencies]\nyetlet-core={version='0.1.0',path='../yetlet-core'}\n",
        ),
        (
            "yetlet-server-axum",
            vec![
                "app.rs",
                "context.rs",
                "document.rs",
                "lib.rs",
                "middleware.rs",
                "response.rs",
            ],
            "[dependencies]\naxum.workspace=true\nserde='1'\ntower='0.5'\nyetlet-core={version='0.1.0',path='../yetlet-core'}\nyetlet-html={version='0.1.0',path='../yetlet-html'}\n[dev-dependencies]\ntokio.workspace=true\nserde={version='1',features=['derive']}\ntower={version='0.5',features=['util']}\ntower-http={version='0.6',features=['set-header']}\nhttp-body='1'\nhttp-body-util='0.1'\n",
        ),
    ] {
        let dir = root.join("crates").join(name);
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::write(dir.join("Cargo.toml"), format!("[package]\nname='{name}'\ndescription='Public fixture package'\nversion.workspace=true\nedition.workspace=true\nrust-version.workspace=true\nlicense.workspace=true\nrepository.workspace=true\n{dependencies}")).unwrap();
        for source in sources {
            fs::write(
                dir.join("src").join(source),
                "// public fixture source only\n",
            )
            .unwrap();
        }
    }
}
