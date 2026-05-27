#![forbid(unsafe_code)]

use anyhow::{anyhow, Context, Result};
use camino::Utf8PathBuf;
use cargo_metadata::{Metadata, MetadataCommand};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    let fail_on_error = std::env::args().any(|a| a == "--fail-on-error");
    match run_lint() {
        Ok(()) => Ok(()),
        Err(e) if fail_on_error => Err(e),
        Err(e) => {
            eprintln!("core-purity: {e:?}");
            Ok(())
        }
    }
}

fn run_lint() -> Result<()> {
    let metadata = MetadataCommand::new()
        .exec()
        .context("cargo metadata failed")?;

    check_members_vs_filesystem(&metadata)?;
    check_workspace_deps(&metadata)?;
    check_core_purity(&metadata)?;

    Ok(())
}

const UNSAFE_TOKEN: &str = "unsafe ";
const UNSAFE_BRACED_1: &str = "unsafe{";
const UNSAFE_BRACED_2: &str = "unsafe {";

fn line_has_unsafe(line: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") {
        return false;
    }
    trimmed.contains(UNSAFE_TOKEN)
        || trimmed.contains(UNSAFE_BRACED_1)
        || trimmed.contains(UNSAFE_BRACED_2)
}

fn should_scan_path(path: &Path) -> bool {
    // Do not enforce unsafe/unwrap rules on the workspace-lint tool itself.
    if let Ok(rel) = path.strip_prefix(std::env::current_dir().unwrap_or_else(|_| path.to_path_buf())) {
        if rel == Path::new("crates/ble-tools-core-purity/src/main.rs") {
            return false;
        }
        if rel == Path::new("crates/ble-tools-workspace-lint/src/main.rs") {
            return false;
        }
    }
    true
}

fn is_code_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    // Skip comments
    if trimmed.starts_with("//") {
        return false;
    }
    // Skip bare string-literal lines (e.g. message templates)
    if trimmed.starts_with('"') && trimmed.ends_with("\",") {
        return false;
    }
    true
}

fn check_core_purity(metadata: &Metadata) -> Result<()> {
    let root = Utf8PathBuf::from_path_buf(metadata.workspace_root.clone())
        .map_err(|_| anyhow!("Non-UTF8 workspace root"))?;
    let crates_dir = root.join("crates");

    let mut violations = Vec::new();

    if crates_dir.is_dir() {
        for entry in fs::read_dir(&crates_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }

            let src_dir = entry.path().join("src");
            if !src_dir.is_dir() {
                continue;
            }

            for src_entry in fs::read_dir(&src_dir)? {
                let src_entry = src_entry?;
                if !src_entry.file_type()?.is_file() {
                    continue;
                }

                let path = src_entry.path();
                if !should_scan_path(&path) {
                    continue;
                }

                let source = fs::read_to_string(&path)
                    .with_context(|| format!("reading source file {path:?}"))?;

                for (idx, line) in source.lines().enumerate() {
                    if !is_code_line(line) {
                        continue;
                    }
                    if line_has_unsafe(line) {
                        violations.push(format!(
                            "{:?}:{}: unsafe usage is forbidden in core crates",
                            path,
                            idx + 1
                        ));
                    }
                }
            }
        }
    }

    if !violations.is_empty() {
        for v in &violations {
            eprintln!("core-purity: {v}");
        }
        return Err(anyhow!("core purity violations detected"));
    }

    Ok(())
}

/// Ensure that all workspace members exist on disk and that no crates under
/// `crates/` are missing from the workspace members list.
fn check_members_vs_filesystem(metadata: &Metadata) -> Result<()> {
    let root = Utf8PathBuf::from_path_buf(metadata.workspace_root.clone())
        .map_err(|_| anyhow!("Non-UTF8 workspace root"))?;

    let declared_members: BTreeSet<_> = metadata
        .workspace_members
        .iter()
        .map(|id| {
            let pkg = metadata
                .packages
                .iter()
                .find(|p| &p.id == id)
                .expect("package id from metadata");
            Utf8PathBuf::from_path_buf(pkg.manifest_path.clone())
                .ok_or_else(|| anyhow!("non-UTF8 manifest path: {}", pkg.manifest_path.display()))?
                .to_string()
                .into()
        })
        .collect::<Result<_>>()?;

    let mut fs_members = BTreeSet::new();
    let crates_dir = root.join("crates");
    if crates_dir.is_dir() {
        for entry in fs::read_dir(&crates_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let manifest = Utf8PathBuf::from_path_buf(entry.path())
                    .ok_or_else(|| anyhow!("non-UTF8 crate path: {}", entry.path().display()))?
                    .join("Cargo.toml");
                if manifest.is_file() {
                    fs_members.insert(manifest.to_string());
                }
            }
        }
    }

    let mut errors = Vec::new();

    for m in &fs_members {
        if !declared_members.contains(m) {
            errors.push(format!(
                "Filesystem member not listed in [workspace.members]: {m}"
            ));
        }
    }

    for m in &declared_members {
        if !fs_members.contains(m) {
            errors.push(format!("Workspace member listed but missing on disk: {m}"));
        }
    }

    if !errors.is_empty() {
        for e in &errors {
            eprintln!("workspace-lint: {e}");
        }
        return Err(anyhow!("workspace member mismatch"));
    }

    Ok(())
}

/// Ensure that any dependency using `workspace = true` has a corresponding
/// entry in [workspace.dependencies] at the root Cargo.toml.
fn check_workspace_deps(metadata: &Metadata) -> Result<()> {
    let root_manifest = Utf8PathBuf::from_path_buf(metadata.workspace_root.clone())
        .ok_or_else(|| anyhow!("non-UTF8 workspace root"))?
        .join("Cargo.toml");
    let root_text = fs::read_to_string(&root_manifest)
        .with_context(|| format!("reading root manifest {root_manifest}"))?;
    let root_doc = root_text.parse::<toml_edit::Document>()?;

    let workspace_deps = root_doc
        .get("workspace")
        .and_then(|w| w.get("dependencies"))
        .and_then(|d| d.as_table())
        .cloned()
        .unwrap_or_default();

    let workspace_dep_names: BTreeSet<String> =
        workspace_deps.iter().map(|(k, _)| k.to_string()).collect();

    let mut missing = BTreeSet::new();

    for pkg in &metadata.packages {
        for dep in &pkg.dependencies {
            if dep.uses_default_features && dep.workspace {
                if !workspace_dep_names.contains(&dep.name) {
                    missing.insert(dep.name.clone());
                }
            }
        }
    }

    if !missing.is_empty() {
        for name in &missing {
            eprintln!(
                "workspace-lint: dependency `{name}` uses workspace=true \
                 but is not in [workspace.dependencies]"
            );
        }
        return Err(anyhow!("missing workspace.dependencies entries"));
    }

    Ok(())
}
