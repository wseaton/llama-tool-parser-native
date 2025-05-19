#![allow(unused)]
use logos::{Lexer, Logos, Span};
use pyo3::prelude::*;
use pythonize::pythonize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

type Error = (String, Span);
type Result<T> = std::result::Result<T, Error>;

/// Simplified Python tokens focusing only on list syntax and function calls with kwargs
#[derive(Debug, Logos, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n\f]+")]
pub enum Token {
    #[token("False", |_| false)]
    #[token("True", |_| true)]
    Bool(bool),

    #[token("<|python_start|>")]
    PythonStart,

    #[token("<|python_end|>")]
    PythonEnd,

    #[token("[")]
    BracketOpen,

    #[token("]")]
    BracketClose,

    #[token("(")]
    ParenOpen,

    #[token(")")]
    ParenClose,

    #[token(",")]
    Comma,

    #[token("=")]
    Equals,

    #[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?", |lex| lex.slice().parse::<f64>().unwrap())]
    Number(f64),

    #[regex(r#"(?:"(?:[^"\\\n]|\\.)*"|'(?:[^'\\\n]|\\.)*')"#, |lex| {
        let s = lex.slice();
        // Remove the quotes
        s[1..s.len()-1].to_owned()
    })]
    String(String),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_owned())]
    Identifier(String),
}

/// Simplified Python AST nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Value {
    Bool(bool),
    Number(f64),
    String(String),
    Identifier(String),
    Empty,
    List(Vec<Value>),
    FunctionCall(FunctionCall),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub kwargs: HashMap<String, Value>,
}

/// Parse the input, specifically formatted for the example text
pub fn parse_python(source: &str) -> Result<Vec<FunctionCall>> {
    let mut outer_list: Vec<FunctionCall> = Vec::new();

    // Use a single approach to find all function calls
    // We'll use the nested function call parser which is more comprehensive
    let inner_functions = parse_nested_function_calls(source)?;
    
    // Extract all function calls and flatten them
    for value in inner_functions.iter() {
        if let Value::FunctionCall(func_call) = value {
            outer_list.push(func_call.clone());
        }
    }

    Ok(outer_list)
}

/// Find all the function calls in the format [function_name(arg="value")]
pub fn parse_nested_function_calls(source: &str) -> Result<Vec<Value>> {
    let mut result = Vec::new();
    let mut lexer = Token::lexer(source);
    let mut in_python_block = false;

    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::PythonStart) => {
                in_python_block = true;
                continue;
            }
            Ok(Token::PythonEnd) => {
                in_python_block = false;
                continue;
            }
            Ok(Token::BracketOpen) => {
                // Only process function calls if we're either inside a Python block 
                // or if we're processing the whole input without Python tokens
                if in_python_block || !source.contains("<|python_start|>") {
                    if let Some(Ok(Token::Identifier(name))) = lexer.next() {
                        if let Some(Ok(Token::ParenOpen)) = lexer.next() {
                            // We found a function call - parse its arguments
                            let func_call = parse_function_with_kwargs(&mut lexer, name)?;
        
                            // Look for closing bracket or Python end token
                            let mut found_close = false;
                            for token in lexer.by_ref().flatten() {
                                if token == Token::BracketClose {
                                    found_close = true;
                                    break;
                                }
                                if token == Token::PythonEnd {
                                    in_python_block = false;
                                    break;
                                }
                            }
        
                            // Add function call
                            result.push(func_call);
                        }
                    }
                }
            }
            _ => continue,
        }
    }

    Ok(result)
}

/// Helper function to handle post-value tokens (comma or closing parenthesis)
pub fn handle_post_value(
    lexer: &mut Lexer<'_, Token>,
    name: String,
    kwargs: HashMap<String, Value>,
) -> Result<Value> {
    match lexer.next() {
        Some(Ok(Token::Comma)) => {
            // Continue to next parameter
            Ok(Value::Empty) // Signal to continue
        }
        Some(Ok(Token::ParenClose)) => {
            // End of arguments
            Ok(Value::FunctionCall(FunctionCall { name, kwargs }))
        }
        _ => {
            // Skip unexpected tokens and continue
            Ok(Value::Empty) // Signal to continue
        }
    }
}

/// Parse a function call with keyword arguments
pub fn parse_function_with_kwargs(lexer: &mut Lexer<'_, Token>, name: String) -> Result<Value> {
    let mut kwargs = HashMap::new();

    loop {
        match lexer.next() {
            Some(Ok(Token::PythonStart)) => {
                // Start of a new Python block
                return Ok(Value::FunctionCall(FunctionCall { name, kwargs }));
            }
            Some(Ok(Token::ParenClose)) => {
                // End of arguments
                return Ok(Value::FunctionCall(FunctionCall { name, kwargs }));
            }
            Some(Ok(Token::Identifier(key))) => {
                // Expect an equals sign
                if let Some(Ok(Token::Equals)) = lexer.next() {
                    // Look for value
                    match lexer.next() {
                        Some(Ok(Token::String(val))) => {
                            kwargs.insert(key, Value::String(val));
                            let result = handle_post_value(lexer, name.clone(), kwargs.clone())?;
                            if let Value::FunctionCall(_) = result {
                                return Ok(result);
                            }
                        }
                        Some(Ok(Token::Bool(val))) => {
                            kwargs.insert(key, Value::Bool(val));
                            let result = handle_post_value(lexer, name.clone(), kwargs.clone())?;
                            if let Value::FunctionCall(_) = result {
                                return Ok(result);
                            }
                        }
                        Some(Ok(Token::Number(val))) => {
                            kwargs.insert(key, Value::Number(val));
                            let result = handle_post_value(lexer, name.clone(), kwargs.clone())?;
                            if let Value::FunctionCall(_) = result {
                                return Ok(result);
                            }
                        }
                        Some(Ok(Token::Identifier(val))) => {
                            kwargs.insert(key, Value::Identifier(val));
                            let result = handle_post_value(lexer, name.clone(), kwargs.clone())?;
                            if let Value::FunctionCall(_) = result {
                                return Ok(result);
                            }
                        }
                        Some(Ok(Token::Comma)) => {
                            // Empty parameter value (key=,)
                            kwargs.insert(key, Value::Empty);
                            // Continue to next parameter
                            continue;
                        }
                        Some(Ok(Token::ParenClose)) => {
                            // Empty parameter at the end (key=))
                            kwargs.insert(key, Value::Empty);
                            return Ok(Value::FunctionCall(FunctionCall { name, kwargs }));
                        }
                        _ => {
                            // For any other token, treat it as an empty value and continue
                            kwargs.insert(key, Value::Empty);
                            continue;
                        }
                    }
                }
            }
            Some(Ok(Token::Comma)) => {
                // Extra comma, continue
                continue;
            }
            Some(Ok(Token::BracketOpen)) => {
                // We've reached a nested list - we're done with this function call
                return Ok(Value::FunctionCall(FunctionCall { name, kwargs }));
            },
            None | Some(Ok(Token::PythonEnd))=> {
                // End of input
                return Ok(Value::FunctionCall(FunctionCall { name, kwargs }));
            }
            _ => {
                // Skip any other tokens
                continue;
            }
        }
    }
}

#[pyfunction(name = "parse_tools")]
pub fn wrapped_parse_python(py: Python, source: String) -> PyResult<Bound<'_, PyAny>> {
    match parse_python(&source) {
        Ok(function_calls) => Ok(pythonize(py, &function_calls).expect("Failed to pythonize")),
        Err((msg, span)) => {
            let error_message = format!("Error at position {}-{}: {}", span.start, span.end, msg);
            Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                error_message,
            ))
        }
    }
}

#[pymodule]
fn llama_tool_parser_native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(wrapped_parse_python, m)?)
}
