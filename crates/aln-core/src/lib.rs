// crates/aln-core/src/lib.rs
//
// MIT OR Apache-2.0
// Rust edition 2024, rust-version = "1.85"
//
// Minimal ALN parsing crate backed by serde_yaml. Assumes ALN text in this
// workspace uses a YAML-compatible subset.

use serde::de::DeserializeOwned;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AlnError {
    #[error("ALN parse error: {0}")]
    Parse(String),
}

/// Parse an ALN text document into a strongly-typed value.
///
/// This assumes that the ALN syntax used in this repository is compatible
/// with YAML when treated as UTF-8 text.
pub fn from_aln_str<T>(input: &str) -> Result<T, AlnError>
where
    T: DeserializeOwned,
{
    serde_yaml::from_str::<T>(input).map_err(|e| AlnError::Parse(e.to_string()))
}

/// Convenience wrapper to parse into serde_yaml::Value for dynamic inspection.
pub fn from_aln_str_value(input: &str) -> Result<serde_yaml::Value, AlnError> {
    from_aln_str::<serde_yaml::Value>(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, serde::Deserialize, PartialEq)]
    struct TestShard {
        version: String,
        #[serde(default)]
        fields: Vec<Field>,
    }

    #[derive(Debug, serde::Deserialize, PartialEq)]
    struct Field {
        name: String,
        #[serde(rename = "type")]
        ty: String,
    }

    #[test]
    fn parses_simple_aln() {
        let aln = r#"
version: "biomem-5d.v1"
fields:
  - name: "rohmaxglobal"
    type: "float"
  - name: "nullspacedim_floors"
    type: "int"
"#;

        let shard: TestShard = from_aln_str(aln).expect("parse ALN");
        assert_eq!(shard.version, "biomem-5d.v1");
        assert_eq!(shard.fields.len(), 2);
    }
}
