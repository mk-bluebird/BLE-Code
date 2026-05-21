// crates/ble-tools-ci-lint-github/src/main.rs

#![forbid(unsafe_code)]

use anyhow::{anyhow, Context, Result};
use glob::glob;
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

fn main() -> Result<()> {
    let fail_on_error = std::env::args().any(|a| a == "--fail-on-error");

    match run_lint() {
        Ok(()) => Ok(()),
        Err(e) if fail_on_error => Err(e),
        Err(e) => {
            eprintln!("ci-lint-github: {e:?}");
            Ok(())
        }
    }
}

fn run_lint() -> Result<()> {
    let mut errors = Vec::new();

    let mut workflow_files = Vec::new();
    for entry in glob(".github/workflows/*.yml")? {
        let path = entry?;
        workflow_files.push(path);
    }

    if workflow_files.is_empty() {
        return Err(anyhow!(
            "no GitHub workflows found under .github/workflows"
        ));
    }

    for path in &workflow_files {
        let text =
            fs::read_to_string(path).with_context(|| format!("reading workflow {}", path.display()))?;
        let doc: serde_yaml::Value =
            serde_yaml::from_str(&text).with_context(|| format!("parsing workflow {}", path.display()))?;

        check_node24_env(path, &doc, &mut errors)?;
        check_approved_actions(path, &doc, &mut errors)?;
        check_required_jobs_present(path, &doc, &mut errors)?;
    }

    if !errors.is_empty() {
        for e in &errors {
            eprintln!("ci-lint-github: {e}");
        }
        return Err(anyhow!("GitHub workflow governance violations"));
    }

    Ok(())
}

fn check_node24_env(path: &Path, doc: &serde_yaml::Value, errors: &mut Vec<String>) -> Result<()> {
    let env = doc.get("env");
    match env {
        Some(serde_yaml::Value::Mapping(map)) => {
            if !map
                .keys()
                .any(|k| k.as_str() == Some("FORCE_JAVASCRIPT_ACTIONS_TO_NODE_24"))
            {
                errors.push(format!(
                    "{}: missing env FORCE_JAVASCRIPT_ACTIONS_TO_NODE_24",
                    path.display()
                ));
            }
        }
        _ => {
            errors.push(format!(
                "{}: missing top-level env with FORCE_JAVASCRIPT_ACTIONS_TO_NODE_24",
                path.display()
            ));
        }
    }
    Ok(())
}

fn check_approved_actions(
    path: &Path,
    doc: &serde_yaml::Value,
    errors: &mut Vec<String>,
) -> Result<()> {
    let approved: BTreeSet<&str> = [
        "actions/checkout@v4",
        "dtolnay/rust-toolchain@v1",
        "actions/setup-java@v4",
        "gradle/actions/setup-gradle@v4",
        "actions/upload-artifact@v4",
        "Swatinem/rust-cache@v2",
    ]
    .into_iter()
    .collect();

    if let Some(jobs) = doc.get("jobs").and_then(|j| j.as_mapping()) {
        for (job_name, job_val) in jobs {
            if let Some(steps) = job_val.get("steps").and_then(|s| s.as_sequence()) {
                for step in steps {
                    if let Some(uses) = step.get("uses").and_then(|u| u.as_str()) {
                        if !approved.contains(uses) {
                            errors.push(format!(
                                "{}: job {} uses unapproved action `{}`",
                                path.display(),
                                job_name.as_str().unwrap_or("<unnamed>"),
                                uses
                            ));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn check_required_jobs_present(
    path: &Path,
    doc: &serde_yaml::Value,
    errors: &mut Vec<String>,
) -> Result<()> {
    let required_jobs: BTreeSet<&str> = [
        "workspace-hygiene",
        "fmt",
        "clippy",
        "test",
        "tools-ci",
        "catalog-ci",
        "perplexity-ci",
        "examples",
        "governance-lint",
    ]
    .into_iter()
    .collect();

    let mut present = BTreeSet::new();
    if let Some(jobs) = doc.get("jobs").and_then(|j| j.as_mapping()) {
        for (job_name, _) in jobs {
            if let Some(name) = job_name.as_str() {
                present.insert(name.to_string());
            }
        }
    }

    for req in &required_jobs {
        if !present.contains(&req.to_string()) {
            errors.push(format!(
                "{}: required job `{}` missing (no-rollback violation)",
                path.display(),
                req
            ));
        }
    }

    Ok(())
}
