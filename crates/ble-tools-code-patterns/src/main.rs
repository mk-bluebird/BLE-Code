// crates/ble-tools-code-patterns/src/main.rs
//
// MIT OR Apache-2.0
// Rust edition 2024, rust-version = "1.85"

use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

#[derive(Debug, Error)]
enum PatternError {
    #[error("IO error at {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("Failed to parse config: {0}")]
    ConfigParse(String),
}

#[derive(Debug, Deserialize)]
struct PatternConfig {
    roots: Vec<String>,
    #[serde(default)]
    ignore_paths: Vec<String>,
    patterns: Vec<PatternRule>,
}

#[derive(Debug, Deserialize)]
struct PatternRule {
    name: String,
    needle: String,
    severity: String, // "error" or "warn"
    message: String,
}

#[derive(Debug, Serialize)]
struct Violation {
    rule: String,
    severity: String,
    path: String,
    line: usize,
    column: usize,
    snippet: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct Report {
    #[serde(rename = "violations")]
    violations: Vec<Violation>,
}

fn main() {
    let exit_code = match run() {
        Ok(report) => {
            let json = serde_json::to_string_pretty(&report).unwrap_or_else(|_| {
                "{\"violations\":[{\"rule\":\"internal-error\",\"severity\":\"error\",\"path\":\"-\",\"line\":0,\"column\":0,\"snippet\":\"\",\"message\":\"failed to render JSON\"}]}".to_string()
            });
            println!("{json}");

            if report
                .violations
                .iter()
                .any(|v| v.severity.eq_ignore_ascii_case("error"))
            {
                1
            } else {
                0
            }
        }
        Err(e) => {
            eprintln!("ble-tools-code-patterns ERROR: {e}");
            1
        }
    };

    std::process::exit(exit_code);
}

fn run() -> Result<Report, PatternError> {
    let cwd = env::current_dir().map_err(|e| PatternError::Io {
        path: ".".to_string(),
        source: e,
    })?;

    let config_path = env::var("BLE_CODE_PATTERNS_CONFIG")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| cwd.join("tools/code_patterns.toml"));

    let cfg = load_config(&config_path)?;
    let mut violations = Vec::new();

    for root in &cfg.roots {
        let root_path = cwd.join(root);
        if !root_path.exists() {
            continue;
        }
        scan_root(&root_path, &cfg, &mut violations)?;
    }

    Ok(Report { violations })
}

fn load_config(path: &Path) -> Result<PatternConfig, PatternError> {
    let contents = fs::read_to_string(path).map_err(|e| PatternError::Io {
        path: path.display().to_string(),
        source: e,
    })?;
    toml::from_str(&contents).map_err(|e| PatternError::ConfigParse(e.to_string()))
}

fn should_ignore(path: &Path, cfg: &PatternConfig) -> bool {
    let s = path.to_string_lossy();
    cfg.ignore_paths.iter().any(|p| s.contains(p))
}

fn scan_root(
    root: &Path,
    cfg: &PatternConfig,
    out: &mut Vec<Violation>,
) -> Result<(), PatternError> {
    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().is_file())
    {
        let path = entry.path().to_path_buf();
        if path.extension().map(|e| e == "rs").unwrap_or(false) && !should_ignore(&path, cfg) {
            scan_file(&path, cfg, out)?;
        }
    }
    Ok(())
}

fn scan_file(
    path: &Path,
    cfg: &PatternConfig,
    out: &mut Vec<Violation>,
) -> Result<(), PatternError> {
    let contents = fs::read_to_string(path).map_err(|e| PatternError::Io {
        path: path.display().to_string(),
        source: e,
    })?;

    for (line_idx, line) in contents.lines().enumerate() {
        let ln = line_idx + 1;
        for rule in &cfg.patterns {
            if rule.needle.is_empty() {
                continue;
            }
            let mut search_start = 0;
            while let Some(pos) = line[search_start..].find(&rule.needle) {
                let col = search_start + pos + 1;
                let snippet = line.trim().to_string();
                out.push(Violation {
                    rule: rule.name.clone(),
                    severity: rule.severity.clone(),
                    path: path.display().to_string(),
                    line: ln,
                    column: col,
                    snippet,
                    message: rule.message.clone(),
                });
                search_start += pos + rule.needle.len();
            }
        }
    }

    Ok(())
}
