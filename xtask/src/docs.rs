//! Documentation language, links, Markdown and backlog DAG checks.
use crate::tooling::*;
use regex::Regex;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};
pub const REQUIRED: &[&str] = &[
    "README.md",
    "AGENTS.md",
    "docs/ARCHITECTURE.md",
    "docs/MVP.md",
    "docs/M2-VERIFICATION.md",
    "docs/M3-VERIFICATION.md",
    "docs/M4-VERIFICATION.md",
    "docs/ROADMAP.md",
    "docs/BACKLOG.md",
    "docs/adr/0001-separation-of-concerns.md",
    "docs/adr/0002-events-and-actions.md",
    "docs/adr/0003-renderers-and-loading.md",
    "docs/adr/0004-request-context-and-middleware.md",
    "docs/adr/0005-opt-in-capability-modules.md",
    "docs/adr/0006-handler-first-mvc-optional.md",
    "docs/adr/0007-bounded-keyed-dom-and-local-graph.md",
    "docs/HTTP-PIPELINE.md",
    "docs/M5-VERIFICATION.md",
    "docs/M6A-VERIFICATION.md",
    "docs/M6-VERIFICATION.md",
    "docs/M7A-VERIFICATION.md",
    "docs/M7B-VERIFICATION.md",
    "docs/M7C-VERIFICATION.md",
    "docs/M7D-VERIFICATION.md",
    "docs/M7E-VERIFICATION.md",
    "docs/M7F-VERIFICATION.md",
    "docs/M7G-VERIFICATION.md",
    "docs/M7H-VERIFICATION.md",
    "docs/M7I-VERIFICATION.md",
    "docs/M7J-VERIFICATION.md",
    "docs/M8A-VERIFICATION.md",
    "docs/M8B-VERIFICATION.md",
    "docs/M8C-VERIFICATION.md",
    "docs/M8D-VERIFICATION.md",
    "docs/M8E-VERIFICATION.md",
    "docs/M9-PLAN.md",
    "docs/M9-PLANNING-VERIFICATION.md",
    "docs/M9A-VERIFICATION.md",
    "docs/M9B-VERIFICATION.md",
    "docs/M9C-VERIFICATION.md",
    "docs/M9D-VERIFICATION.md",
    "docs/M9E-DATA-ACCESS-COMPARISON.md",
    "docs/M9E-VERIFICATION.md",
    "docs/M9F-VERIFICATION.md",
    "docs/M9G-VERIFICATION.md",
    "docs/M9H-VERIFICATION.md",
    "docs/M9I-VERIFICATION.md",
    "docs/M9J-VERIFICATION.md",
    "docs/API-CONTRACTS.md",
    "docs/CI-SECURITY-REVIEW.md",
    "docs/M10C-PLAN.md",
    "docs/M10C-PLANNING-VERIFICATION.md",
    "docs/COMPATIBILITY.md",
    "docs/M10C-VERIFICATION.md",
    "docs/M10D-PLAN.md",
    "docs/M10D-PLANNING-VERIFICATION.md",
    "docs/THREAT-MODEL.md",
    "docs/M10D-VERIFICATION.md",
    "docs/M10A-VERIFICATION.md",
    "docs/M10B-VERIFICATION.md",
    "docs/M10-REMAINING-PLAN.md",
    "docs/RELEASE-SUPPLY-CHAIN.md",
    "docs/M10E-VERIFICATION.md",
    "docs/QUALIFICATION-MATRIX.md",
    "docs/M10F-VERIFICATION.md",
    "docs/PROVIDER-OPERATIONS.md",
    "docs/M10G-VERIFICATION.md",
    "docs/EXTERNAL-SECURITY-REVIEW.md",
    "docs/M10H-VERIFICATION.md",
    "docs/M10-DELIVERY-VERIFICATION.md",
    "CONTRIBUTING.md",
    "SECURITY.md",
    "docs/SAMPLES.md",
    "docs/EXTENSIONS.md",
    "docs/RELEASE-READINESS.md",
];
const SKIP: &[&str] = &[".git", ".codex", "target", "dist", "node_modules", ".cache"];
pub fn check(root: &Path) -> Result<Vec<String>> {
    let mut failures = Vec::new();
    let link = Regex::new(r"\]\(([^)]+)\)").unwrap();
    for name in REQUIRED {
        let path = safe(root, name, false)?;
        if !path.is_file() {
            failures.push(format!("missing: {name}"));
            continue;
        }
        let text = text(root, name)?;
        if !text.ends_with('\n') {
            failures.push(format!("missing EOF newline: {name}"));
        }
        if text
            .lines()
            .filter(|l| l.trim_start().starts_with("```"))
            .count()
            % 2
            != 0
        {
            failures.push(format!("unbalanced code fences: {name}"));
        }
        for (index, line) in text.lines().enumerate() {
            if line.trim_end() != line {
                failures.push(format!("trailing whitespace: {name}:{}", index + 1));
            }
        }
        for target in link.captures_iter(&text) {
            let target = &target[1];
            let file = target.split('#').next().unwrap();
            if file.is_empty()
                || ["https://", "http://", "mailto:"]
                    .iter()
                    .any(|prefix| file.starts_with(prefix))
            {
                continue;
            }
            if !path.parent().unwrap().join(file).is_file() {
                failures.push(format!("broken link: {name} -> {target}"));
            }
        }
    }
    english(root, root, &mut failures)?;
    if !root.join("docs/BACKLOG.md").is_file() {
        return Ok(failures);
    }
    let task = Regex::new(r"^\|\s*(YT-\d{3})\s*\|").unwrap();
    let dep = Regex::new(r"YT-\d{3}").unwrap();
    let mut rows = BTreeMap::new();
    for line in text(root, "docs/BACKLOG.md")?.lines() {
        if let Some(cap) = task.captures(line) {
            let id = cap[1].to_owned();
            if rows.contains_key(&id) {
                failures.push(format!("duplicate backlog ID: {id}"));
            }
            rows.insert(
                id,
                line.split('|')
                    .map(|s| s.trim().to_owned())
                    .collect::<Vec<_>>(),
            );
        }
    }
    if rows.is_empty() {
        failures.push("no backlog work items".into());
    }
    let mut edges = BTreeMap::new();
    for (id, fields) in &rows {
        if fields.len() < 9 {
            failures.push(format!("malformed backlog row: {id}"));
            continue;
        }
        let deps = dep
            .find_iter(&fields[6])
            .map(|s| s.as_str().to_owned())
            .collect::<Vec<_>>();
        for dependency in &deps {
            if !rows.contains_key(dependency) {
                failures.push(format!("unknown backlog dependency: {id} -> {dependency}"));
            }
            if dependency == id {
                failures.push(format!("self dependency: {id}"));
            }
        }
        edges.insert(id.clone(), deps);
    }
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for id in rows.keys() {
        walk(id, &edges, &mut visiting, &mut visited, &mut failures);
    }
    println!(
        "Documents: {}, backlog items: {}",
        REQUIRED.len(),
        rows.len()
    );
    Ok(failures)
}
fn walk(
    id: &str,
    edges: &BTreeMap<String, Vec<String>>,
    visiting: &mut BTreeSet<String>,
    visited: &mut BTreeSet<String>,
    failures: &mut Vec<String>,
) {
    if visiting.contains(id) {
        failures.push(format!("dependency cycle involving: {id}"));
        return;
    }
    if visited.contains(id) {
        return;
    }
    visiting.insert(id.into());
    if let Some(deps) = edges.get(id) {
        for dep in deps {
            if edges.contains_key(dep) {
                walk(dep, edges, visiting, visited, failures);
            }
        }
    }
    visiting.remove(id);
    visited.insert(id.into());
}
fn english(root: &Path, dir: &Path, failures: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        // Do not follow directory symlinks; bounded traversal remains inside the source root.
        if file_type.is_dir() {
            if !SKIP.iter().any(|s| entry.file_name() == *s) {
                english(root, &path, failures)?;
            }
        } else if path.extension().is_some_and(|s| s == "md") {
            let relative = path
                .strip_prefix(root)
                .map_err(|_| fail("document escapes root"))?;
            let text = text(root, relative)?;
            for (i, line) in text.lines().enumerate() {
                if line
                    .chars()
                    .any(|c| matches!(c,'\u{3400}'..='\u{9fff}'|'\u{3000}'..='\u{303f}'))
                {
                    failures.push(format!(
                        "non-English CJK content: {}:{}",
                        relative.display(),
                        i + 1
                    ));
                }
            }
        }
    }
    Ok(())
}
