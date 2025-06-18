//! Rust type signature parser.
//!
//! Parses string representations of Rust types into RustType enum.

use super::types::RustType;

/// Simple parser for Rust type signatures
/// This is a basic implementation - a full parser would need proper tokenization
pub struct RustTypeParser;

impl RustTypeParser {
    /// Parse a simple Rust type string
    pub fn parse(type_str: &str) -> Result<RustType, String> {
        let trimmed = type_str.trim();

        // Handle references
        if let Some(rest) = trimmed.strip_prefix("&mut ") {
            return Ok(RustType::MutRef {
                lifetime: None,
                inner: Box::new(Self::parse(rest)?),
            });
        }

        if let Some(rest) = trimmed.strip_prefix("&") {
            // Check for lifetime
            let (lifetime, rest) = if rest.starts_with('\'') {
                let end = rest.find(' ').unwrap_or(rest.len());
                let lifetime = rest[1..end].to_string();
                let remaining = if end < rest.len() {
                    rest[end..].trim()
                } else {
                    ""
                };
                (Some(lifetime), remaining)
            } else {
                (None, rest)
            };

            return Ok(RustType::Ref {
                lifetime,
                inner: Box::new(Self::parse(rest)?),
            });
        }

        // Handle Box<T>
        if let Some(inner) = trimmed
            .strip_prefix("Box<")
            .and_then(|s| s.strip_suffix('>'))
        {
            return Ok(RustType::Box(Box::new(Self::parse(inner)?)));
        }

        // Handle Vec<T>
        if let Some(inner) = trimmed
            .strip_prefix("Vec<")
            .and_then(|s| s.strip_suffix('>'))
        {
            return Ok(RustType::Vec(Box::new(Self::parse(inner)?)));
        }

        // Handle Option<T>
        if let Some(inner) = trimmed
            .strip_prefix("Option<")
            .and_then(|s| s.strip_suffix('>'))
        {
            return Ok(RustType::Option(Box::new(Self::parse(inner)?)));
        }

        // Handle Result<T, E>
        if let Some(inner) = trimmed
            .strip_prefix("Result<")
            .and_then(|s| s.strip_suffix('>'))
        {
            // Split by comma, but need to handle nested generics
            let mut depth = 0;
            let mut split_pos = None;
            for (i, ch) in inner.chars().enumerate() {
                match ch {
                    '<' => depth += 1,
                    '>' => depth -= 1,
                    ',' if depth == 0 => {
                        split_pos = Some(i);
                        break;
                    }
                    _ => {}
                }
            }

            if let Some(pos) = split_pos {
                let ok_type = inner[..pos].trim();
                let err_type = inner[pos + 1..].trim();
                return Ok(RustType::Result {
                    ok: Box::new(Self::parse(ok_type)?),
                    err: Box::new(Self::parse(err_type)?),
                });
            } else {
                return Err("Invalid Result type: missing error type".to_string());
            }
        }

        // Handle basic types
        match trimmed {
            "i32" => Ok(RustType::I32),
            "i64" => Ok(RustType::I64),
            "isize" => Ok(RustType::ISize),
            "u32" => Ok(RustType::U32),
            "u64" => Ok(RustType::U64),
            "usize" => Ok(RustType::USize),
            "bool" => Ok(RustType::Bool),
            "char" => Ok(RustType::Char),
            "()" => Ok(RustType::Unit),
            "!" => Ok(RustType::Never),
            "str" => Ok(RustType::Str),
            "String" => Ok(RustType::String),
            _ => {
                // Assume it's a custom type or generic parameter
                if trimmed.len() == 1 && trimmed.chars().next().map_or(false, |c| c.is_uppercase())
                {
                    Ok(RustType::Generic(trimmed.to_string()))
                } else {
                    Ok(RustType::Custom {
                        name: trimmed.to_string(),
                        generics: vec![],
                    })
                }
            }
        }
    }
}
