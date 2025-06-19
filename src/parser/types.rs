//! Type parsing for the Veltrano language
//!
//! This module contains all type parsing logic including:
//! - Primitive types (integers, bool, char, etc.)
//! - Reference types (Ref, Own, MutRef)
//! - Container types (Box, Vec, Array, Option, Result)
//! - Custom types

use super::Parser;
use crate::ast_types::Located;
use crate::error::{SourceLocation, Span, VeltranoError};
use crate::lexer::TokenType;
use crate::types::VeltranoType;

impl Parser {
    pub(super) fn parse_type(&mut self) -> Result<Located<VeltranoType>, VeltranoError> {
        let start_token = self.peek();
        let start_location = SourceLocation::new(start_token.line, start_token.column);
        let vtype = self.parse_type_inner()?;
        let end_token = self.previous();
        let end_location = SourceLocation::new(end_token.line, end_token.column);
        Ok(Located::new(vtype, Span::new(start_location, end_location)))
    }

    fn parse_type_inner(&mut self) -> Result<VeltranoType, VeltranoError> {
        if let TokenType::Identifier(type_name) = &self.peek().token_type {
            let type_name = type_name.clone();
            self.advance();

            match type_name.as_str() {
                // Signed integers
                "I32" => Ok(VeltranoType::i32()),
                "I64" => Ok(VeltranoType::i64()),
                "ISize" => Ok(VeltranoType::isize()),
                // Unsigned integers
                "U32" => Ok(VeltranoType::u32()),
                "U64" => Ok(VeltranoType::u64()),
                "USize" => Ok(VeltranoType::usize()),
                // Other primitives
                "Bool" => Ok(VeltranoType::bool()),
                "Char" => Ok(VeltranoType::char()),
                "Unit" => Ok(VeltranoType::unit()),
                "Nothing" => Ok(VeltranoType::nothing()),
                // String types
                "Str" => Ok(VeltranoType::str()), // naturally referenced
                "String" => Ok(VeltranoType::string()), // naturally referenced
                "Ref" => self.parse_ref_type(),
                "Own" => self.parse_own_type(),
                "MutRef" => self.parse_mutref_type(),
                "Box" => self.parse_box_type(),
                "Vec" => self.parse_vec_type(),
                "Array" => self.parse_array_type(),
                "Option" => self.parse_option_type(),
                "Result" => self.parse_result_type(),
                _ => Ok(VeltranoType::custom(type_name)), // naturally referenced
            }
        } else {
            Err(self.syntax_error("Expected type name".to_string()))
        }
    }

    fn parse_ref_type(&mut self) -> Result<VeltranoType, VeltranoError> {
        self.consume(&TokenType::Less, "Expected '<' after Ref")?;
        let inner_type = self.parse_type()?;
        self.consume(&TokenType::Greater, "Expected '>' after type parameter")?;
        Ok(VeltranoType::ref_(inner_type.node))
    }

    fn parse_own_type(&mut self) -> Result<VeltranoType, VeltranoError> {
        self.consume(&TokenType::Less, "Expected '<' after Own")?;
        let inner_type = self.parse_type()?;
        self.consume(&TokenType::Greater, "Expected '>' after type parameter")?;

        // Validation is now handled by the type checker
        Ok(VeltranoType::own(inner_type.node))
    }

    fn parse_mutref_type(&mut self) -> Result<VeltranoType, VeltranoError> {
        self.consume(&TokenType::Less, "Expected '<' after MutRef")?;
        let inner_type = self.parse_type()?;
        self.consume(&TokenType::Greater, "Expected '>' after type parameter")?;
        Ok(VeltranoType::mut_ref(inner_type.node))
    }

    fn parse_box_type(&mut self) -> Result<VeltranoType, VeltranoError> {
        self.consume(&TokenType::Less, "Expected '<' after Box")?;
        let inner_type = self.parse_type()?;
        self.consume(&TokenType::Greater, "Expected '>' after type parameter")?;
        Ok(VeltranoType::boxed(inner_type.node))
    }

    fn parse_vec_type(&mut self) -> Result<VeltranoType, VeltranoError> {
        self.consume(&TokenType::Less, "Expected '<' after Vec")?;
        let inner_type = self.parse_type()?;
        self.consume(&TokenType::Greater, "Expected '>' after type parameter")?;
        Ok(VeltranoType::vec(inner_type.node))
    }

    fn parse_array_type(&mut self) -> Result<VeltranoType, VeltranoError> {
        self.consume(&TokenType::Less, "Expected '<' after Array")?;
        let inner_type = self.parse_type()?;
        self.consume(&TokenType::Comma, "Expected ',' after array element type")?;

        // Parse array size
        if let TokenType::IntLiteral(size) = &self.peek().token_type {
            let size = *size as usize;
            self.advance();
            self.consume(&TokenType::Greater, "Expected '>' after array size")?;
            Ok(VeltranoType::array(inner_type.node, size))
        } else {
            Err(self.syntax_error("Expected integer literal for array size".to_string()))
        }
    }

    fn parse_option_type(&mut self) -> Result<VeltranoType, VeltranoError> {
        self.consume(&TokenType::Less, "Expected '<' after Option")?;
        let inner_type = self.parse_type()?;
        self.consume(&TokenType::Greater, "Expected '>' after type parameter")?;
        Ok(VeltranoType::option(inner_type.node))
    }

    fn parse_result_type(&mut self) -> Result<VeltranoType, VeltranoError> {
        self.consume(&TokenType::Less, "Expected '<' after Result")?;
        let ok_type = self.parse_type()?;
        self.consume(&TokenType::Comma, "Expected ',' after Result ok type")?;
        let err_type = self.parse_type()?;
        self.consume(&TokenType::Greater, "Expected '>' after Result error type")?;
        Ok(VeltranoType::result(ok_type.node, err_type.node))
    }
}
