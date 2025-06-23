//! Syn-based querier for Rust type information.
//!
//! Uses the syn crate to parse Rust source files and extract type information.

use super::cache::*;
use super::parser::RustTypeParser;
use super::types::*;
use super::{RustInteropError, RustQuerier};
use crate::error::VeltranoError;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Default crate version when not found in metadata
const DEFAULT_CRATE_VERSION: &str = "0.1.0";

/// Queries Rust type information by parsing source files with syn
#[derive(Debug)]
pub struct SynQuerier {
    cargo_metadata: Option<CargoMetadata>,
    project_root: PathBuf,
}

#[derive(Debug, Clone)]
struct CargoMetadata {
    packages: Vec<PackageMetadata>,
    _workspace_root: PathBuf,
}

#[derive(Debug, Clone)]
struct PackageMetadata {
    name: String,
    version: String,
    _manifest_path: PathBuf,
    targets: Vec<TargetMetadata>,
}

#[derive(Debug, Clone)]
struct TargetMetadata {
    _name: String,
    kind: Vec<String>,
    src_path: PathBuf,
}

impl SynQuerier {
    /// Exposed for testing only. Not part of the stable public API.
    #[doc(hidden)]
    #[allow(dead_code)]
    pub fn extract_from_source(&self, crate_name: &str) -> Result<CrateInfo, VeltranoError> {
        let crate_root = self.find_crate_root(crate_name).ok_or_else(|| {
            VeltranoError::from(RustInteropError::CrateNotFound(crate_name.to_string()))
        })?;
        let file = self.parse_rust_file(&crate_root)?;
        self.extract_crate_info(crate_name, &file)
    }

    /// Exposed for testing only. Not part of the stable public API.
    #[doc(hidden)]
    #[allow(dead_code)]
    pub fn parse_function(&self, func: &syn::ItemFn) -> Result<FunctionInfo, VeltranoError> {
        self.extract_function(func, "test_crate")
    }

    /// Exposed for testing only. Not part of the stable public API.
    #[doc(hidden)]
    #[allow(dead_code)]
    pub fn parse_struct(&self, s: &syn::ItemStruct) -> Result<TypeInfo, VeltranoError> {
        self.extract_struct(s, "test_crate")
    }

    /// Exposed for testing only. Not part of the stable public API.
    #[doc(hidden)]
    #[allow(dead_code)]
    pub fn parse_enum(&self, e: &syn::ItemEnum) -> Result<TypeInfo, VeltranoError> {
        self.extract_enum(e, "test_crate")
    }

    /// Exposed for testing only. Not part of the stable public API.
    #[doc(hidden)]
    #[allow(dead_code)]
    pub fn parse_trait(&self, t: &syn::ItemTrait) -> Result<TraitInfo, VeltranoError> {
        self.extract_trait(t, "test_crate")
    }

    /// Exposed for testing only. Not part of the stable public API.
    #[doc(hidden)]
    #[allow(dead_code)]
    pub fn parse_impl_block(
        &self,
        impl_block: &syn::ItemImpl,
        crate_info: &mut CrateInfo,
    ) -> Result<(), VeltranoError> {
        self.process_impl_block(impl_block, crate_info)
    }

    /// Exposed for testing only. Not part of the stable public API.
    #[doc(hidden)]
    #[allow(dead_code)]
    pub fn convert_syn_type_to_rust_type(&self, ty: &syn::Type) -> Option<RustType> {
        self.syn_type_to_signature(ty).parsed
    }

    /// Exposed for testing only. Not part of the stable public API.
    #[doc(hidden)]
    #[allow(dead_code)]
    pub fn determine_self_kind(
        &self,
        inputs: &syn::punctuated::Punctuated<syn::FnArg, syn::token::Comma>,
    ) -> SelfKind {
        if let Some(syn::FnArg::Receiver(receiver)) = inputs.first() {
            if receiver.reference.is_some() {
                if receiver.mutability.is_some() {
                    SelfKind::MutRef
                } else {
                    SelfKind::Ref
                }
            } else {
                SelfKind::Value
            }
        } else {
            SelfKind::None
        }
    }

    pub fn new(project_root: Option<PathBuf>) -> Result<Self, VeltranoError> {
        let project_root =
            project_root.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

        // Try to get cargo metadata
        let cargo_metadata = Self::get_cargo_metadata(&project_root)?;

        Ok(Self {
            cargo_metadata,
            project_root,
        })
    }

    fn get_cargo_metadata(project_root: &Path) -> Result<Option<CargoMetadata>, VeltranoError> {
        let output = Command::new("cargo")
            .args(&["metadata", "--format-version", "1"])
            .current_dir(project_root)
            .output()
            .map_err(|e| {
                VeltranoError::from(RustInteropError::CargoError(format!(
                    "Failed to run cargo metadata: {}",
                    e
                )))
            })?;

        if !output.status.success() {
            // Not in a cargo project, that's okay
            return Ok(None);
        }

        let metadata_json = String::from_utf8(output.stdout).map_err(|e| {
            VeltranoError::from(RustInteropError::ParseError(format!(
                "Invalid UTF-8 in cargo metadata: {}",
                e
            )))
        })?;

        let metadata: serde_json::Value = serde_json::from_str(&metadata_json).map_err(|e| {
            VeltranoError::from(RustInteropError::ParseError(format!(
                "Invalid cargo metadata JSON: {}",
                e
            )))
        })?;

        // Extract relevant information
        let packages = metadata["packages"].as_array().ok_or_else(|| {
            VeltranoError::from(RustInteropError::ParseError(
                "No packages in metadata".to_string(),
            ))
        })?;

        let workspace_root = metadata["workspace_root"].as_str().ok_or_else(|| {
            VeltranoError::from(RustInteropError::ParseError(
                "No workspace_root in metadata".to_string(),
            ))
        })?;

        let mut package_list = Vec::new();
        for package in packages {
            let name = package["name"].as_str().unwrap_or_default().to_string();
            let version = package["version"].as_str().unwrap_or_default().to_string();
            let manifest_path =
                PathBuf::from(package["manifest_path"].as_str().unwrap_or_default());

            let mut targets = Vec::new();
            if let Some(target_array) = package["targets"].as_array() {
                for target in target_array {
                    let target_name = target["name"].as_str().unwrap_or_default().to_string();
                    let kinds = target["kind"]
                        .as_array()
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default();
                    let src_path = PathBuf::from(target["src_path"].as_str().unwrap_or_default());

                    targets.push(TargetMetadata {
                        _name: target_name,
                        kind: kinds,
                        src_path,
                    });
                }
            }

            package_list.push(PackageMetadata {
                name,
                version,
                _manifest_path: manifest_path,
                targets,
            });
        }

        Ok(Some(CargoMetadata {
            packages: package_list,
            _workspace_root: PathBuf::from(workspace_root),
        }))
    }

    fn find_crate_root(&self, crate_name: &str) -> Option<PathBuf> {
        if let Some(ref metadata) = self.cargo_metadata {
            // Look for the crate in cargo metadata
            for package in &metadata.packages {
                if package.name == crate_name {
                    // Find lib target
                    for target in &package.targets {
                        if target.kind.contains(&"lib".to_string()) {
                            return Some(target.src_path.clone());
                        }
                    }
                    // Fallback to any target
                    if let Some(target) = package.targets.first() {
                        return Some(target.src_path.clone());
                    }
                }
            }
        }

        // Fallback: look for src/lib.rs in project root
        let lib_path = self.project_root.join("src").join("lib.rs");
        if lib_path.exists() {
            Some(lib_path)
        } else {
            None
        }
    }

    fn parse_rust_file(&self, path: &Path) -> Result<syn::File, VeltranoError> {
        let content = fs::read_to_string(path).map_err(|e| {
            VeltranoError::from(RustInteropError::IoError(format!(
                "Failed to read {}: {}",
                path.display(),
                e
            )))
        })?;

        syn::parse_file(&content).map_err(|e| {
            VeltranoError::from(RustInteropError::ParseError(format!(
                "Failed to parse {}: {}",
                path.display(),
                e
            )))
        })
    }

    fn extract_crate_info(
        &self,
        crate_name: &str,
        file: &syn::File,
    ) -> Result<CrateInfo, VeltranoError> {
        let mut crate_info = CrateInfo {
            name: crate_name.to_string(),
            version: self
                .cargo_metadata
                .as_ref()
                .and_then(|m| m.packages.iter().find(|p| p.name == crate_name))
                .map(|p| p.version.clone())
                .unwrap_or_else(|| DEFAULT_CRATE_VERSION.to_string()),
            functions: HashMap::new(),
            types: HashMap::new(),
            traits: HashMap::new(),
            trait_implementations: HashMap::new(),
        };

        // Process items in the file
        for item in &file.items {
            match item {
                syn::Item::Fn(func) => {
                    if let Ok(func_info) = self.extract_function(func, crate_name) {
                        crate_info
                            .functions
                            .insert(func.sig.ident.to_string(), func_info);
                    }
                }
                syn::Item::Struct(s) => {
                    if let Ok(type_info) = self.extract_struct(s, crate_name) {
                        crate_info.types.insert(s.ident.to_string(), type_info);
                    }
                }
                syn::Item::Enum(e) => {
                    if let Ok(type_info) = self.extract_enum(e, crate_name) {
                        crate_info.types.insert(e.ident.to_string(), type_info);
                    }
                }
                syn::Item::Trait(t) => {
                    if let Ok(trait_info) = self.extract_trait(t, crate_name) {
                        crate_info.traits.insert(t.ident.to_string(), trait_info);
                    }
                }
                syn::Item::Impl(impl_block) => {
                    let _ = self.process_impl_block(impl_block, &mut crate_info);
                }
                _ => {}
            }
        }

        Ok(crate_info)
    }

    /// Exposed for testing only. Not part of the stable public API.
    #[doc(hidden)]
    pub fn extract_function(
        &self,
        func: &syn::ItemFn,
        crate_name: &str,
    ) -> Result<FunctionInfo, VeltranoError> {
        let func_name = func.sig.ident.to_string();
        Ok(FunctionInfo {
            name: func_name.clone(),
            path: RustPath::ModuleItem(
                RustModulePath(crate_name.into(), vec![]),
                func_name,
                ItemKind::Function,
            ),
            generics: self.extract_generics(&func.sig.generics),
            parameters: self.extract_parameters(&func.sig),
            return_type: self.extract_return_type(&func.sig.output),
            is_unsafe: func.sig.unsafety.is_some(),
            is_const: func.sig.constness.is_some(),
            documentation: None, // TODO: Extract doc comments
        })
    }

    /// Exposed for testing only. Not part of the stable public API.
    #[doc(hidden)]
    pub fn extract_struct(
        &self,
        s: &syn::ItemStruct,
        crate_name: &str,
    ) -> Result<TypeInfo, VeltranoError> {
        let fields = match &s.fields {
            syn::Fields::Named(fields) => fields
                .named
                .iter()
                .filter_map(|f| {
                    let name = f.ident.as_ref()?.to_string();
                    Some(FieldInfo {
                        name,
                        field_type: self.syn_type_to_signature(&f.ty),
                        is_public: matches!(f.vis, syn::Visibility::Public(_)),
                    })
                })
                .collect(),
            _ => vec![],
        };

        let type_name = s.ident.to_string();
        Ok(TypeInfo {
            name: type_name.clone(),
            path: RustPath::Type(RustTypePath(
                RustModulePath(crate_name.into(), vec![]),
                vec![type_name],
            )),
            kind: TypeKind::Struct,
            generics: self.extract_generics(&s.generics),
            methods: vec![],
            fields,
            variants: vec![],
        })
    }

    /// Exposed for testing only. Not part of the stable public API.
    #[doc(hidden)]
    pub fn extract_enum(
        &self,
        e: &syn::ItemEnum,
        crate_name: &str,
    ) -> Result<TypeInfo, VeltranoError> {
        let variants = e
            .variants
            .iter()
            .map(|v| {
                let fields = match &v.fields {
                    syn::Fields::Named(fields) => {
                        fields
                            .named
                            .iter()
                            .filter_map(|f| {
                                let name = f.ident.as_ref()?.to_string();
                                Some(FieldInfo {
                                    name,
                                    field_type: self.syn_type_to_signature(&f.ty),
                                    is_public: true, // Enum variant fields are always accessible
                                })
                            })
                            .collect()
                    }
                    _ => vec![],
                };

                VariantInfo {
                    name: v.ident.to_string(),
                    fields,
                }
            })
            .collect();

        let type_name = e.ident.to_string();
        Ok(TypeInfo {
            name: type_name.clone(),
            path: RustPath::Type(RustTypePath(
                RustModulePath(crate_name.into(), vec![]),
                vec![type_name],
            )),
            kind: TypeKind::Enum,
            generics: self.extract_generics(&e.generics),
            methods: vec![],
            fields: vec![],
            variants,
        })
    }

    /// Exposed for testing only. Not part of the stable public API.
    #[doc(hidden)]
    pub fn extract_trait(
        &self,
        t: &syn::ItemTrait,
        crate_name: &str,
    ) -> Result<TraitInfo, VeltranoError> {
        let methods = t
            .items
            .iter()
            .filter_map(|item| {
                if let syn::TraitItem::Fn(method) = item {
                    Some(MethodInfo {
                        name: method.sig.ident.to_string(),
                        self_kind: self.extract_self_kind(&method.sig),
                        generics: self.extract_generics(&method.sig.generics),
                        parameters: self.extract_parameters(&method.sig),
                        return_type: self.extract_return_type(&method.sig.output),
                        is_unsafe: method.sig.unsafety.is_some(),
                        is_const: method.sig.constness.is_some(),
                    })
                } else {
                    None
                }
            })
            .collect();

        let trait_name = t.ident.to_string();
        Ok(TraitInfo {
            name: trait_name.clone(),
            path: RustPath::Type(RustTypePath(
                RustModulePath(crate_name.into(), vec![]),
                vec![trait_name],
            )),
            methods,
            associated_types: vec![], // TODO: Extract associated types
        })
    }

    /// Exposed for testing only. Not part of the stable public API.
    #[doc(hidden)]
    pub fn process_impl_block(
        &self,
        impl_block: &syn::ItemImpl,
        crate_info: &mut CrateInfo,
    ) -> Result<(), VeltranoError> {
        // Extract type being implemented for
        let type_name = if let syn::Type::Path(type_path) = &*impl_block.self_ty {
            type_path
                .path
                .segments
                .last()
                .map(|seg| seg.ident.to_string())
        } else {
            None
        };

        let Some(type_name) = type_name else {
            return Ok(());
        };

        // Check if this is a trait implementation
        if let Some((_, trait_path, _)) = &impl_block.trait_ {
            let trait_name = trait_path
                .segments
                .last()
                .map(|seg| seg.ident.to_string())
                .unwrap_or_default();

            crate_info
                .trait_implementations
                .entry(type_name.clone())
                .or_insert_with(HashSet::new)
                .insert(trait_name);
        }

        // Extract methods
        let methods: Vec<MethodInfo> = impl_block
            .items
            .iter()
            .filter_map(|item| {
                if let syn::ImplItem::Fn(method) = item {
                    Some(MethodInfo {
                        name: method.sig.ident.to_string(),
                        self_kind: self.extract_self_kind(&method.sig),
                        generics: self.extract_generics(&method.sig.generics),
                        parameters: self.extract_parameters(&method.sig),
                        return_type: self.extract_return_type(&method.sig.output),
                        is_unsafe: method.sig.unsafety.is_some(),
                        is_const: method.sig.constness.is_some(),
                    })
                } else {
                    None
                }
            })
            .collect();

        // Add methods to the type
        if let Some(type_info) = crate_info.types.get_mut(&type_name) {
            type_info.methods.extend(methods);
        }

        Ok(())
    }

    fn extract_generics(&self, generics: &syn::Generics) -> Vec<GenericParam> {
        generics
            .params
            .iter()
            .filter_map(|param| {
                if let syn::GenericParam::Type(type_param) = param {
                    let bounds = type_param
                        .bounds
                        .iter()
                        .filter_map(|bound| {
                            if let syn::TypeParamBound::Trait(trait_bound) = bound {
                                Some(self.path_to_string(&trait_bound.path))
                            } else {
                                None
                            }
                        })
                        .collect();

                    Some(GenericParam {
                        name: type_param.ident.to_string(),
                        bounds,
                        default: None, // TODO: Extract default
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    fn extract_parameters(&self, sig: &syn::Signature) -> Vec<Parameter> {
        sig.inputs
            .iter()
            .filter_map(|arg| {
                if let syn::FnArg::Typed(pat_type) = arg {
                    let name = if let syn::Pat::Ident(ident) = &*pat_type.pat {
                        ident.ident.to_string()
                    } else {
                        "_".to_string()
                    };

                    Some(Parameter {
                        name,
                        param_type: self.syn_type_to_signature(&pat_type.ty),
                    })
                } else {
                    None // Skip self parameter
                }
            })
            .collect()
    }

    fn extract_self_kind(&self, sig: &syn::Signature) -> SelfKind {
        if let Some(syn::FnArg::Receiver(receiver)) = sig.inputs.first() {
            if receiver.reference.is_some() {
                if receiver.mutability.is_some() {
                    SelfKind::MutRef // TODO: Extract lifetime
                } else {
                    SelfKind::Ref // TODO: Extract lifetime
                }
            } else {
                SelfKind::Value
            }
        } else {
            SelfKind::None
        }
    }

    fn extract_return_type(&self, output: &syn::ReturnType) -> RustTypeSignature {
        match output {
            syn::ReturnType::Default => RustTypeSignature {
                raw: "()".to_string(),
                parsed: Some(RustType::Unit),
                lifetimes: vec![],
                bounds: vec![],
            },
            syn::ReturnType::Type(_, ty) => self.syn_type_to_signature(ty),
        }
    }

    fn syn_type_to_signature(&self, ty: &syn::Type) -> RustTypeSignature {
        let raw = quote::quote!(#ty).to_string();
        let parsed = RustTypeParser::parse(&raw).ok();

        RustTypeSignature {
            raw,
            parsed,
            lifetimes: vec![], // TODO: Extract lifetimes
            bounds: vec![],    // TODO: Extract bounds
        }
    }

    fn path_to_string(&self, path: &syn::Path) -> String {
        path.segments
            .iter()
            .map(|seg| seg.ident.to_string())
            .collect::<Vec<_>>()
            .join("::")
    }
}

impl RustQuerier for SynQuerier {
    fn query_crate(&mut self, crate_name: &str) -> Result<CrateInfo, VeltranoError> {
        let crate_root = self.find_crate_root(crate_name).ok_or_else(|| {
            VeltranoError::from(RustInteropError::CrateNotFound(crate_name.to_string()))
        })?;

        let file = self.parse_rust_file(&crate_root)?;
        self.extract_crate_info(crate_name, &file)
    }

    fn supports_crate(&self, _crate_name: &str) -> bool {
        self.cargo_metadata.is_some()
    }

    fn priority(&self) -> u32 {
        80 // Lower than rustdoc but higher than rust-analyzer
    }
}
