//! Finite browser-only starter, using the existing protected publisher.
use crate::scaffold;

pub(crate) fn files(name: &str, tailwind: bool) -> Result<Vec<(&'static str, String)>, String> {
    scaffold::validate_name(name).map_err(str::to_owned)?;
    let mut package: serde_json::Value = serde_json::from_str(if tailwind {
        include_str!("../templates/vite/tailwind-package.json")
    } else {
        include_str!("../templates/vite/plain-package.json")
    })
    .map_err(|_| "invalid built-in frontend package")?;
    package["scripts"]["build"] = "node build-wasm.mjs && vite build".into();
    // Run previews prepared output; explicit browser watch selects Vite development.
    package["scripts"]["dev"] = "node serve.mjs".into();
    let vite = include_str!("../templates/browser/vite.config.mjs");
    let vite = if tailwind {
        vite.replace(
            "// TAILWIND_IMPORT",
            "import tailwindcss from '@tailwindcss/vite';",
        )
        .replace("/* TAILWIND_PLUGIN */", "tailwindcss(),")
    } else {
        vite.into()
    };
    let version = crate::framework::package_version();
    Ok(vec![
        ("Cargo.toml", format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\nrust-version = \"1.99\"\npublish = false\n\n[workspace]\nresolver = \"3\"\n\n[lib]\nname = \"tasks\"\ncrate-type = [\"cdylib\", \"rlib\"]\n\n[target.'cfg(target_arch = \"wasm32\")'.dependencies]\nyetlet = {{ version = \"={version}\", default-features = false, features = [\"reactive\"] }}\nyetlet-browser = {{ version = \"={version}\", default-features = false, features = [\"dom\", \"dom-keyed\"] }}\nwasm-bindgen = \"=0.2.129\"\nweb-sys = {{ version = \"0.3\", features = [\"Document\", \"Element\", \"Event\", \"EventTarget\", \"HtmlElement\", \"HtmlInputElement\", \"Node\", \"Window\"] }}\n")),
        ("README.md", include_str!("../templates/browser/README.md").replace("@NAME@", name)),
        (".gitignore", "/target/\n/frontend/node_modules/\n/frontend/dist/\n/frontend/generated/\n/frontend/.env*\n!/frontend/.env.example\n".into()),
        ("base.toml", "[cli]\ntarget = \"browser\"\n\n[cli.rust]\nwatch = false\n\n[cli.vite]\nenabled = true\nroot = \"frontend/\"\nauto_start = true\nauto_build = true\n\n[cli.generate.k8s]\nreplicas = 1\n".into()),
        ("development.toml", "# Public CLI overrides inherit base.toml.\n".into()),
        ("staging.toml", "# Public CLI overrides inherit base.toml.\n".into()),
        ("production.toml", "# Public CLI overrides inherit base.toml.\n".into()),
        ("src/lib.rs", include_str!("../templates/browser/lib.rs").into()),
        ("src/model.rs", include_str!("../templates/browser/model.rs").into()),
        ("frontend/index.html", include_str!("../templates/browser/index.html").into()),
        ("frontend/app.ts", include_str!("../templates/browser/app.ts").into()),
        ("frontend/wasm-startup.mjs", include_str!("../../../assets/wasm-startup.mjs").into()),
        ("frontend/wasm-startup.css", include_str!("../../../assets/wasm-startup.css").into()),
        ("frontend/app.css", format!("{}{}", if tailwind { "@import \"tailwindcss\";\n@source \"../src/lib.rs\";\n" } else { "" }, include_str!("../templates/browser/app.css"))),
        ("frontend/serve.mjs", include_str!("../templates/browser/serve.mjs").into()),
        ("frontend/build-wasm.mjs", include_str!("../templates/browser/build-wasm.mjs").into()),
        ("frontend/vite.config.mjs", vite),
        ("frontend/tsconfig.json", include_str!("../templates/vite/tsconfig.json").into()),
        ("frontend/mark.svg", "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 32 32\"><path fill=\"#4338ca\" d=\"M2 3h7l7 12 7-12h7L19 22v8h-6v-8z\"/></svg>\n".into()),
        ("frontend/package.json", format!("{}\n", serde_json::to_string_pretty(&package).map_err(|_| "invalid built-in frontend package")?)),
        ("frontend/package-lock.json", if tailwind { include_str!("../templates/vite/tailwind-package-lock.json") } else { include_str!("../templates/vite/plain-package-lock.json") }.into()),
        ("frontend/.env.example", "# Only public VITE_* values belong here. Never put credentials in browser configuration.\nVITE_WORKSPACE_LABEL=Local workspace\n".into()),
    ])
}

#[cfg(test)]
mod startup_tests {
    #[test]
    fn browser_startup_assets_and_pre_wasm_links_are_emitted() {
        let files = super::files("startup-contract", false).unwrap();
        let get = |name| {
            files
                .iter()
                .find(|(path, _)| *path == name)
                .unwrap()
                .1
                .as_str()
        };
        assert_eq!(
            get("frontend/wasm-startup.mjs"),
            include_str!("../../../assets/wasm-startup.mjs")
        );
        assert_eq!(
            get("frontend/wasm-startup.css"),
            include_str!("../../../assets/wasm-startup.css")
        );
        let html = get("frontend/index.html");
        assert!(
            html.find("href=\"./wasm-startup.css\"").unwrap()
                < html.find("src=\"./app.ts\"").unwrap()
        );
        assert!(
            html.contains("data-startup-state=\"loading\"") && html.contains("reload this page")
        );
        let entry = get("frontend/app.ts");
        assert!(!entry.contains("import init,"));
        assert!(entry.contains("await import('./generated/tasks.js')"));
        assert!(
            get("frontend/serve.mjs")
                .contains("code.replace(\"await import('./generated/tasks.js')\"")
        );
    }
}
