//! Rustdoc-based querier for Rust type information.
//!
//! Uses rustdoc JSON output to extract type information from crates.

use super::cache::*;
use super::parser::RustTypeParser;
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
            name: doc.crate_name.clone().unwrap_or_default(),
            version: doc.crate_version.clone().unwrap_or_default(),
            functions: HashMap::new(),
            types: HashMap::new(),
            traits: HashMap::new(),
            trait_implementations: HashMap::new(),
        };

        // Process all items in the index
        for (_id, item) in &doc.index {
            match item.kind.as_str() {
                "function" | "constant" | "static" => {
                    if let Some(func_info) = self.convert_function(&item, &doc) {
                        crate_info.functions.insert(item.name.clone(), func_info);
                    }
                }
                "struct" | "enum" | "union" => {
                    if let Some(type_info) = self.convert_type(&item, &doc) {
                        crate_info.types.insert(item.name.clone(), type_info);
                    }
                }
                "trait" => {
                    if let Some(trait_info) = self.convert_trait(&item, &doc) {
                        crate_info.traits.insert(item.name.clone(), trait_info);
                    }
                }
                _ => {}
            }
        }

        Ok(crate_info)
    }

    fn convert_function(&self, item: &RustdocItem, doc: &RustdocJson) -> Option<FunctionInfo> {
        // Extract function details from rustdoc JSON
        let inner = item.inner.as_ref()?;
        let func: RustdocFunction = serde_json::from_value(inner.clone()).ok()?;

        // Build the path for this function/constant/static
        // Determine the correct ItemKind based on the item type
        let item_kind = match item.kind.as_str() {
            "function" => ItemKind::Function,
            "constant" => ItemKind::Constant,
            "static" => ItemKind::Static,
            _ => return None, // Shouldn't happen due to match in caller
        };

        // Extract the crate name and module path from paths
        let (crate_name, module_path) = if let Some(item_summary) = doc.paths.get(&item.id) {
            // Get crate name - either from current crate or external crates
            let crate_name = if item.crate_id == 0 {
                doc.crate_name
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string())
            } else {
                doc.external_crates
                    .get(&item.crate_id)
                    .map(|ec| ec.name.clone())
                    .unwrap_or_else(|| "unknown".to_string())
            };

            // Extract module path by removing crate name and item name from the full path
            let mut module_path = item_summary.path.clone();
            if !module_path.is_empty() {
                module_path.remove(0); // Remove crate name
            }
            if !module_path.is_empty() && module_path.last() == Some(&item.name) {
                module_path.pop(); // Remove item name
            }

            (crate_name, module_path)
        } else {
            // Fallback if paths entry is missing
            (
                doc.crate_name
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
                vec![],
            )
        };

        let path = RustPath::ModuleItem(
            RustModulePath(CrateName(crate_name), module_path),
            item.name.clone(),
            item_kind,
        );

        // Convert parameters
        let parameters = func
            .sig
            .inputs
            .into_iter()
            .map(|(name, type_str)| Parameter {
                name,
                param_type: RustTypeSignature {
                    raw: type_str.clone(),
                    parsed: RustTypeParser::parse(&type_str).ok(),
                    lifetimes: vec![],
                    bounds: vec![],
                },
            })
            .collect();

        // Convert return type
        let return_type_str = func.sig.output.unwrap_or_else(|| "()".to_string());
        let return_type = RustTypeSignature {
            raw: return_type_str.clone(),
            parsed: RustTypeParser::parse(&return_type_str).ok(),
            lifetimes: vec![],
            bounds: vec![],
        };

        // Convert generics
        let generics = func
            .generics
            .params
            .into_iter()
            .map(|param| GenericParam {
                name: param.name,
                bounds: param.bounds,
                default: param.default,
            })
            .collect();

        Some(FunctionInfo {
            name: item.name.clone(),
            path,
            generics,
            parameters,
            return_type,
            is_unsafe: func.header.is_unsafe,
            is_const: func.header.is_const,
            documentation: None, // TODO: Extract docs if available
        })
    }

    fn convert_type(&self, item: &RustdocItem, doc: &RustdocJson) -> Option<TypeInfo> {
        // Extract type details from rustdoc JSON
        let inner = item.inner.as_ref()?;

        // Determine TypeKind from item.kind
        let type_kind = match item.kind.as_str() {
            "struct" => TypeKind::Struct,
            "enum" => TypeKind::Enum,
            "union" => TypeKind::Union,
            _ => return None, // Not a type we handle here
        };

        // Parse the inner structure based on type kind
        let (fields, variants, generics) = match item.kind.as_str() {
            "struct" => {
                let struct_data: RustdocStruct = serde_json::from_value(inner.clone()).ok()?;
                (struct_data.fields, vec![], struct_data.generics)
            }
            "enum" => {
                let enum_data: RustdocEnum = serde_json::from_value(inner.clone()).ok()?;
                (vec![], enum_data.variants, enum_data.generics)
            }
            "union" => {
                let union_data: RustdocUnion = serde_json::from_value(inner.clone()).ok()?;
                (union_data.fields, vec![], union_data.generics)
            }
            _ => return None,
        };

        // Extract the crate name and module path from paths
        let (crate_name, module_path) = if let Some(item_summary) = doc.paths.get(&item.id) {
            // Get crate name - either from current crate or external crates
            let crate_name = if item.crate_id == 0 {
                doc.crate_name
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string())
            } else {
                doc.external_crates
                    .get(&item.crate_id)
                    .map(|ec| ec.name.clone())
                    .unwrap_or_else(|| "unknown".to_string())
            };

            // Extract module path by removing crate name and item name from the full path
            let mut module_path = item_summary.path.clone();
            if !module_path.is_empty() {
                module_path.remove(0); // Remove crate name
            }
            if !module_path.is_empty() && module_path.last() == Some(&item.name) {
                module_path.pop(); // Remove item name
            }

            (crate_name, module_path)
        } else {
            // Fallback if paths entry is missing
            (
                doc.crate_name
                    .clone()
                    .unwrap_or_else(|| "unknown".to_string()),
                vec![],
            )
        };

        // Build the path for this type
        let path = RustPath::Type(RustTypePath(
            RustModulePath(CrateName(crate_name), module_path),
            vec![item.name.clone()],
        ));

        // Convert fields
        let fields = fields
            .into_iter()
            .map(|field| FieldInfo {
                name: field.name,
                field_type: RustTypeSignature {
                    raw: field.type_str.clone(),
                    parsed: RustTypeParser::parse(&field.type_str).ok(),
                    lifetimes: vec![],
                    bounds: vec![],
                },
                is_public: field.is_public,
            })
            .collect();

        // Convert variants
        let variants = variants
            .into_iter()
            .map(|variant| VariantInfo {
                name: variant.name,
                fields: variant
                    .fields
                    .into_iter()
                    .map(|field| FieldInfo {
                        name: field.name,
                        field_type: RustTypeSignature {
                            raw: field.type_str.clone(),
                            parsed: RustTypeParser::parse(&field.type_str).ok(),
                            lifetimes: vec![],
                            bounds: vec![],
                        },
                        is_public: true, // Enum variant fields are always accessible
                    })
                    .collect(),
            })
            .collect();

        // Convert generics
        let generics = generics
            .params
            .into_iter()
            .map(|param| GenericParam {
                name: param.name,
                bounds: param.bounds,
                default: param.default,
            })
            .collect();

        Some(TypeInfo {
            name: item.name.clone(),
            path,
            kind: type_kind,
            generics,
            methods: vec![], // TODO: Extract methods when available
            fields,
            variants,
        })
    }

    fn convert_trait(&self, _item: &RustdocItem, _doc: &RustdocJson) -> Option<TraitInfo> {
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
    paths: HashMap<String, RustdocItemSummary>,
    external_crates: HashMap<u32, RustdocExternalCrate>,
}

#[derive(Debug, Deserialize)]
struct RustdocItem {
    id: String,
    crate_id: u32,
    name: String,
    kind: String,
    #[allow(dead_code)]
    inner: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct RustdocItemSummary {
    crate_id: u32,
    path: Vec<String>,
    kind: String,
}

#[derive(Debug, Deserialize)]
struct RustdocExternalCrate {
    name: String,
    html_root_url: Option<String>,
}

// Rustdoc function representation
#[derive(Debug, Deserialize)]
struct RustdocFunction {
    sig: RustdocFunctionSignature,
    generics: RustdocGenerics,
    header: RustdocFunctionHeader,
}

#[derive(Debug, Deserialize)]
struct RustdocFunctionSignature {
    inputs: Vec<(String, String)>, // (param_name, type_string)
    output: Option<String>,        // Return type as string
}

#[derive(Debug, Deserialize)]
struct RustdocFunctionHeader {
    is_const: bool,
    is_unsafe: bool,
    is_async: bool,
}

#[derive(Debug, Deserialize)]
struct RustdocGenerics {
    params: Vec<RustdocGenericParam>,
}

#[derive(Debug, Deserialize)]
struct RustdocGenericParam {
    name: String,
    bounds: Vec<String>,
    default: Option<String>,
}

// Rustdoc struct representation
#[derive(Debug, Deserialize)]
struct RustdocStruct {
    fields: Vec<RustdocField>,
    generics: RustdocGenerics,
}

// Rustdoc enum representation
#[derive(Debug, Deserialize)]
struct RustdocEnum {
    variants: Vec<RustdocVariant>,
    generics: RustdocGenerics,
}

// Rustdoc union representation
#[derive(Debug, Deserialize)]
struct RustdocUnion {
    fields: Vec<RustdocField>,
    generics: RustdocGenerics,
}

#[derive(Debug, Deserialize)]
struct RustdocField {
    name: String,
    #[serde(rename = "type")]
    type_str: String,
    #[serde(default)]
    is_public: bool,
}

#[derive(Debug, Deserialize)]
struct RustdocVariant {
    name: String,
    fields: Vec<RustdocField>,
}
