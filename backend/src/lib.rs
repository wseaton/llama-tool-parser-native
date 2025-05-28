#![allow(unused)]
use pyo3::prelude::*;
use pythonize::pythonize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Import the parsers
mod logos_parser;
pub mod nom_parser;

// Re-export the parsers
pub use logos_parser::parse_python;
pub use nom_parser::{NomParserState, parse_incremental, parse_python_with_nom};

// Re-export the Error and Result types from logos parser
pub use logos_parser::{Error, Result};

/// Simplified Python AST nodes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Value {
    Bool(bool),
    Number(f64),
    String(String),
    Identifier(String),
    Empty,
    List(Vec<Value>),
    FunctionCall(FunctionCall),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionCall {
    pub name: String,
    pub kwargs: HashMap<String, Value>,
}
