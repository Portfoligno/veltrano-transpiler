//! Rustdoc-based querier for Rust type information.
//!
//! Uses rustdoc JSON output to extract type information from crates.

use super::cache::*;
use super::{RustInteropError, RustQuerier};
use crate::error::VeltranoError;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Queries Rust type information using rustdoc JSON output
#[derive(Debug)]
pub struct RustdocQuerier {
    cache_dir: PathBuf,
}

impl RustdocQuerier {
    /// Exposed for testing only. Not part of the stable public API.
    #[doc(hidden)]
    #[allow(dead_code)]
    pub fn extract_crate_info(&self, crate_name: &str) -> Result<CrateInfo, VeltranoError> {
        let json_path = self.generate_rustdoc_json(crate_name)?;
        self.parse_rustdoc_json(&json_path)
    }

    pub fn new(cache_dir: Option<PathBuf>) -> Self {
        Self {
            cache_dir: cache_dir
                .unwrap_or_else(|| std::env::temp_dir().join("veltrano_rustdoc_cache")),
        }
    }

    fn generate_rustdoc_json(&self, crate_name: &str) -> Result<PathBuf, VeltranoError> {
        // Create cache directory if it doesn't exist
        fs::create_dir_all(&self.cache_dir)
            .map_err(|e| VeltranoError::from(RustInteropError::IoError(e.to_string())))?;

        let json_path = self.cache_dir.join(format!("{}.json", crate_name));

        // Check if cached version exists and is recent
        if json_path.exists() {
            if let Ok(metadata) = fs::metadata(&json_path) {
                if let Ok(modified) = metadata.modified() {
                    if let Ok(elapsed) = modified.elapsed() {
                        // Cache for 24 hours
                        if elapsed.as_secs() < 86400 {
                            return Ok(json_path);
                        }
                    }
                }
            }
        }

        // Generate new rustdoc JSON
        let output = Command::new("rustdoc")
            .args(&[
                "-Z",
                "unstable-options",
                "--output-format",
                "json",
                "--crate-name",
                crate_name,
                "-",
            ])
            .output()
            .map_err(|e| {
                VeltranoError::from(RustInteropError::CargoError(format!(
                    "Failed to run rustdoc: {}",
                    e
                )))
            })?;

        if !output.status.success() {
            return Err(VeltranoError::from(RustInteropError::CargoError(format!(
                "rustdoc failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ))));
        }

        fs::write(&json_path, output.stdout)
            .map_err(|e| VeltranoError::from(RustInteropError::IoError(e.to_string())))?;

        Ok(json_path)
    }

    fn parse_rustdoc_json(&self, json_path: &Path) -> Result<CrateInfo, VeltranoError> {
        let json_content = fs::read_to_string(json_path)
            .map_err(|e| VeltranoError::from(RustInteropError::IoError(e.to_string())))?;

        // Parse the rustdoc JSON format
        let doc: RustdocJson = serde_json::from_str(&json_content).map_err(|e| {
            VeltranoError::from(RustInteropError::ParseError(format!(
                "Invalid rustdoc JSON: {}",
                e
            )))
        })?;

        // Convert rustdoc format to our CrateInfo format
        self.convert_rustdoc_to_crate_info(doc)
    }

    fn convert_rustdoc_to_crate_info(&self, doc: RustdocJson) -> Result<CrateInfo, VeltranoError> {
        let mut crate_info = CrateInfo {
            name: doc.crate_name.unwrap_or_default(),
            version: doc.crate_version.unwrap_or_default(),
            functions: HashMap::new(),
            types: HashMap::new(),
            traits: HashMap::new(),
            trait_implementations: HashMap::new(),
        };

        // Process all items in the index
        for (_id, item) in doc.index {
            match item.kind.as_str() {
                "function" => {
                    if let Some(func_info) = self.convert_function(&item) {
                        crate_info.functions.insert(item.name.clone(), func_info);
                    }
                }
                "struct" | "enum" | "union" => {
                    if let Some(type_info) = self.convert_type(&item) {
                        crate_info.types.insert(item.name.clone(), type_info);
                    }
                }
                "trait" => {
                    if let Some(trait_info) = self.convert_trait(&item) {
                        crate_info.traits.insert(item.name.clone(), trait_info);
                    }
                }
                _ => {}
            }
        }

        Ok(crate_info)
    }

    fn convert_function(&self, _item: &RustdocItem) -> Option<FunctionInfo> {
        // TODO: Implement proper rustdoc function parsing
        None
    }

    fn convert_type(&self, _item: &RustdocItem) -> Option<TypeInfo> {
        // TODO: Implement proper rustdoc type parsing
        None
    }

    fn convert_trait(&self, _item: &RustdocItem) -> Option<TraitInfo> {
        // TODO: Implement proper rustdoc trait parsing
        None
    }
}

impl RustQuerier for RustdocQuerier {
    fn query_crate(&mut self, crate_name: &str) -> Result<CrateInfo, VeltranoError> {
        // For now, only support std library with hardcoded data
        if crate_name == "std" {
            // Return empty for now, StdLibQuerier handles std
            return Err(VeltranoError::from(RustInteropError::CrateNotFound(
                crate_name.to_string(),
            )));
        }

        let json_path = self.generate_rustdoc_json(crate_name)?;
        self.parse_rustdoc_json(&json_path)
    }

    fn supports_crate(&self, crate_name: &str) -> bool {
        // In theory supports any crate, but practically limited
        crate_name == "std" || crate_name.starts_with("core")
    }

    fn priority(&self) -> u32 {
        90 // High priority for standard library docs
    }
}

// Rustdoc JSON format structures (simplified)
#[derive(Debug, Deserialize)]
struct RustdocJson {
    crate_name: Option<String>,
    crate_version: Option<String>,
    index: HashMap<String, RustdocItem>,
}

#[derive(Debug, Deserialize)]
struct RustdocItem {
    name: String,
    kind: String,
    #[allow(dead_code)]
    inner: Option<serde_json::Value>,
}
