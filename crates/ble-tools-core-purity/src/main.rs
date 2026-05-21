// crates/ble-tools-core-purity/src/main.rs
//
// MIT OR Apache-2.0
// Rust edition: 2024, rust-version = "1.85"
//
// Core purity checker for BLE "core" crates.
// Checks:
//   - No forbidden dependencies (config-driven).
//   - No platform-specific cfgs or modules.
//   - No unsafe usage in core crates.
//   - No direct BLE actuation calls (config-driven patterns).

use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;
use toml::Value as TomlValue;
use walkdir::WalkDir;

#[derive(Debug, Error)]
enum PurityError {
    #[error("IO error at {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("Failed to parse config: {0}")]
    ConfigParse(String),
    #[error("Failed to parse Cargo.toml at {0}: {1}")]
    CargoParse(String, String),
}

#[derive(Debug, Deserialize)]
struct CorePurityConfig {
    /// Workspace-relative paths of "core" crates to check, e.g. ["crates/ble-model", "crates/ble-governance"].
    core_crates: Vec<String>,

    /// Crate names that must not appear as direct dependencies of core crates.
    forbidden_deps: Vec<String>,

    /// String patterns that indicate platform-specific code (cfgs, modules, or imports).
    platform_patterns: Vec<String>,

    /// String patterns that indicate direct BLE actuation, e.g. "ble_hw::actuate".
    forbidden_actuation_patterns: Vec<String>,

    /// Optional: additional Rust source globs to include per crate (relative to crate root).
    #[serde(default)]
    extra_src_globs: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Default)]
struct Violation {
    crate_name: String,
    path: PathBuf,
    line: usize,
    message: String,
}

fn main() {
    let exit_code = match run() {
        Ok(violations) => {
            if violations.is_empty() {
                println!("[core-purity] OK: no violations found");
                0
            } else {
                eprintln!("[core-purity] Found {} violation(s):", violations.len());
                for v in &violations {
                    eprintln!(
                        "- {}:{}: {} :: {}",
                        v.path.display(),
                        v.line,
                        v.crate_name,
                        v.message
                    );
                }
                1
            }
        }
        Err(err) => {
            eprintln!("[core-purity] ERROR: {err}");
            1
        }
    };

    std::process::exit(exit_code);
}

fn run() -> Result<Vec<Violation>, PurityError> {
    let cwd = env::current_dir().map_err(|e| PurityError::Io {
        path: ".".to_string(),
        source: e,
    })?;

    let config_path = env::var("BLE_CORE_PURITY_CONFIG")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| cwd.join("tools/core_purity.toml"));

    let cfg = load_config(&config_path)?;
    let mut violations = Vec::new();

    for rel in &cfg.core_crates {
        let crate_root = cwd.join(rel);
        let cargo_toml = crate_root.join("Cargo.toml");
        let crate_name = match read_crate_name(&cargo_toml) {
            Ok(name) => name,
            Err(e) => {
                return Err(e);
            }
        };

        // 1. Dependency purity
        let deps = read_deps(&cargo_toml)?;
        check_forbidden_deps(
            &crate_name,
            &cargo_toml,
            &deps,
            &cfg.forbidden_deps,
            &mut violations,
        );

        // 2. Source-level checks
        let src_dir = crate_root.join("src");
        check_sources(
            &crate_name,
            &src_dir,
            &cfg.platform_patterns,
            &cfg.forbidden_actuation_patterns,
            &mut violations,
        )?;
    }

    Ok(violations)
}

fn load_config(path: &Path) -> Result<CorePurityConfig, PurityError> {
    let contents = fs::read_to_string(path).map_err(|e| PurityError::Io {
        path: path.display().to_string(),
        source: e,
    })?;
    toml::from_str(&contents).map_err(|e| {
        PurityError::ConfigParse(format!("{} (in {})", e, path.display()))
    })
}

fn read_crate_name(cargo_toml: &Path) -> Result<String, PurityError> {
    let contents = fs::read_to_string(cargo_toml).map_err(|e| PurityError::Io {
        path: cargo_toml.display().to_string(),
        source: e,
    })?;
    let value: TomlValue = contents
        .parse()
        .map_err(|e| PurityError::CargoParse(cargo_toml.display().to_string(), e.to_string()))?;

    let pkg = value
        .get("package")
        .and_then(|p| p.as_table())
        .ok_or_else(|| {
            PurityError::CargoParse(
                cargo_toml.display().to_string(),
                "missing [package]".to_string(),
            )
        })?;

    let name = pkg
        .get("name")
        .and_then(|n| n.as_str())
        .ok_or_else(|| {
            PurityError::CargoParse(
                cargo_toml.display().to_string(),
                "missing package.name".to_string(),
            )
        })?;

    Ok(name.to_string())
}

fn read_deps(cargo_toml: &Path) -> Result<Vec<String>, PurityError> {
    let contents = fs::read_to_string(cargo_toml).map_err(|e| PurityError::Io {
        path: cargo_toml.display().to_string(),
        source: e,
    })?;
    let value: TomlValue = contents
        .parse()
        .map_err(|e| PurityError::CargoParse(cargo_toml.display().to_string(), e.to_string()))?;

    let mut out = Vec::new();
    for section in &["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(table) = value.get(*section).and_then(|v| v.as_table()) {
            for (k, _v) in table {
                out.push(k.clone());
            }
        }
    }
    Ok(out)
}

fn check_forbidden_deps(
    crate_name: &str,
    cargo_toml: &Path,
    deps: &[String],
    forbidden: &[String],
    out: &mut Vec<Violation>,
) {
    for fd in forbidden {
        if deps.iter().any(|d| d == fd) {
            out.push(Violation {
                crate_name: crate_name.to_string(),
                path: cargo_toml.to_path_buf(),
                line: 1,
                message: format!("forbidden dependency '{fd}' in core crate"),
            });
        }
    }
}

fn check_sources(
    crate_name: &str,
    src_root: &Path,
    platform_patterns: &[String],
    forbidden_actuation: &[String],
    out: &mut Vec<Violation>,
) -> Result<(), PurityError> {
    if !src_root.exists() {
        return Ok(());
    }

    for entry in WalkDir::new(src_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file() && e.path().extension().map(|e| e == "rs").unwrap_or(false))
    {
        let path = entry.into_path();
        let content = fs::read_to_string(&path).map_err(|e| PurityError::Io {
            path: path.display().to_string(),
            source: e,
        })?;

        // Split once so we can compute line numbers cheaply.
        for (idx, line) in content.lines().enumerate() {
            let ln = idx + 1;

            // Unsafe usage beyond crate attributes.
            if line.contains("unsafe ") || line.contains("unsafe{") || line.contains("unsafe {") {
                // Heuristic: ignore crate-level attributes like #![forbid(unsafe_code)]
                if !line.trim_start().starts_with("#!") {
                    out.push(Violation {
                        crate_name: crate_name.to_string(),
                        path: path.clone(),
                        line: ln,
                        message: "unsafe usage is forbidden in core crates".to_string(),
                    });
                }
            }

            // Platform-specific patterns.
            for pat in platform_patterns {
                if !pat.is_empty() && line.contains(pat) {
                    out.push(Violation {
                        crate_name: crate_name.to_string(),
                        path: path.clone(),
                        line: ln,
                        message: format!("platform-specific pattern '{pat}' not allowed in core crate"),
                    });
                }
            }

            // Direct BLE actuation calls.
            for pat in forbidden_actuation {
                if !pat.is_empty() && line.contains(pat) {
                    out.push(Violation {
                        crate_name: crate_name.to_string(),
                        path: path.clone(),
                        line: ln,
                        message: format!("direct BLE actuation via '{pat}' not allowed in core crate"),
                    });
                }
            }
        }
    }

    Ok(())
}
