#![forbid(unsafe_code)]

use anyhow::{anyhow, Context, Result};
use cargo_metadata::{Metadata, MetadataCommand, Package};
use camino::Utf8PathBuf;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;

fn main() -> Result<()> {
    let fail_on_error = std::env::args().any(|a| a == "--fail-on-error");
    match run_lint() {
        Ok(()) => Ok(()),
        Err(e) if fail_on_error => Err(e),
        Err(e) => {
            eprintln!("workspace-lint: {e:?}");
            Ok(())
        }
    }
}

fn run_lint() -> Result<()> {
    let metadata = MetadataCommand::new().exec().context("cargo metadata failed")?;

    check_members_vs_filesystem(&metadata)?;
    check_workspace_deps(&metadata)?;

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
                .unwrap()
                .to_string()
        })
        .collect();

    let mut fs_members = BTreeSet::new();
    let crates_dir = root.join("crates");
    if crates_dir.is_dir() {
        for entry in fs::read_dir(&crates_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let manifest = Utf8PathBuf::from_path_buf(entry.path())
                    .unwrap()
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
            errors.push(format!("Filesystem member not listed in [workspace.members]: {m}"));
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
        .unwrap()
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
