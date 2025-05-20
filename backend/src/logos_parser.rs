use logos::{Lexer, Logos, Span};
use std::collections::HashMap;

use crate::{FunctionCall, Value};

pub type Error = (String, Span);
pub type Result<T> = std::result::Result<T, Error>;

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

/// Parse the input, specifically formatted for the example text
pub fn parse_python(source: &str) -> Result<Vec<FunctionCall>> {
    let mut outer_list: Vec<FunctionCall> = Vec::new();

    // Use a single approach to find all function calls
    // We'll use the nested function call parser which is more comprehensive
    let inner_functions = parse_nested_function_calls(source)?;
    tracing::debug!(
        "Results from nested function calls: {} items",
        inner_functions.len()
    );

    // Extract all function calls and flatten them
    for (i, value) in inner_functions.iter().enumerate() {
        tracing::debug!("Processing result {}: {:?}", i, value);
        if let Value::FunctionCall(func_call) = value {
            outer_list.push(func_call.clone());
            tracing::debug!("Added function call: {}", func_call.name);
        }
    }

    tracing::debug!("Final result has {} items", outer_list.len());
    Ok(outer_list)
}

/// Find all the function calls in the format [function_name(arg="value")]
/// Also handles comma-separated lists of function calls: [func1(arg1="val1"), func2(arg2="val2")]
pub fn parse_nested_function_calls(source: &str) -> Result<Vec<Value>> {
    tracing::debug!("\n---- PARSE_NESTED_FUNCTION_CALLS ----");
    tracing::debug!("Source: {}", source);
    // Reset for the actual parsing
    let mut result = Vec::new();
    let mut lexer = Token::lexer(source);
    let mut in_python_block = false;

    // Simplified approach for direct function list parsing
    // Find opening bracket first
    while let Some(token) = lexer.next() {
        match token {
            Ok(Token::PythonStart) => {
                tracing::debug!("Found PythonStart");
                in_python_block = true;
            }
            Ok(Token::BracketOpen) => {
                tracing::debug!("Found BracketOpen - parsing function list");

                // Process the first function
                if let Some(first_func) = parse_next_function_in_list(&mut lexer)? {
                    tracing::debug!("Parsed first function: {:?}", first_func);
                    result.push(first_func);

                    // Now look for comma-separated additional functions
                    loop {
                        match lexer.next() {
                            Some(Ok(Token::Comma)) => {
                                tracing::debug!("Found comma between functions");
                                // After comma, try to parse another function
                                if let Some(next_func) = parse_next_function_in_list(&mut lexer)? {
                                    tracing::debug!("Parsed additional function: {:?}", next_func);
                                    result.push(next_func);
                                } else {
                                    tracing::debug!("No function after comma");
                                    break;
                                }
                            }
                            Some(Ok(Token::BracketClose)) => {
                                tracing::debug!("Found BracketClose - end of function list");
                                break;
                            }
                            Some(Ok(Token::PythonEnd)) => {
                                tracing::debug!("Found PythonEnd");
                                in_python_block = false;
                                break;
                            }
                            Some(other) => {
                                tracing::debug!("Unexpected token between functions: {:?}", other);
                                break;
                            }
                            None => {
                                tracing::debug!("End of input in function list");
                                break;
                            }
                        }
                    }
                }
            }
            Ok(Token::PythonEnd) => {
                tracing::debug!("Found PythonEnd");
                in_python_block = false;
            }
            _ => {} // Skip other tokens
        }
    }

    tracing::debug!("Final result size: {}", result.len());
    Ok(result)
}

/// Parse a single function from the token stream, starting at the function name
fn parse_next_function_in_list(lexer: &mut Lexer<'_, Token>) -> Result<Option<Value>> {
    // First token should be an identifier (function name)
    match lexer.next() {
        Some(Ok(Token::Identifier(name))) => {
            tracing::debug!("Found function name: {}", name);

            // Next should be opening parenthesis
            match lexer.next() {
                Some(Ok(Token::ParenOpen)) => {
                    tracing::debug!("Found opening parenthesis for {}", name);
                    // Parse function arguments
                    let func_call = parse_function_with_kwargs(lexer, name)?;
                    Ok(Some(func_call))
                }
                other => {
                    tracing::debug!("Expected opening parenthesis, got: {:?}", other);
                    Ok(None) // Not a function call
                }
            }
        }
        other => {
            tracing::debug!("Expected identifier (function name), got: {:?}", other);
            Ok(None) // Not a function call
        }
    }
}

/// Helper function to parse multiple function calls within a list
fn parse_function_calls_in_list(
    lexer: &mut Lexer<'_, Token>,
    result: &mut Vec<Value>,
    in_python_block: &mut bool,
) -> Result<()> {
    // Process all function calls in the list until we hit the closing bracket
    loop {
        // Find the next identifier which should be a function name
        let mut found_function = false;

        tracing::debug!("Looking for next function name...");
        while let Some(token) = lexer.next() {
            tracing::debug!("Token: {:?}", token);
            match token {
                Ok(Token::BracketClose) => {
                    tracing::debug!("Found BracketClose");
                    // End of the list, exit the function
                    return Ok(());
                }
                Ok(Token::PythonEnd) => {
                    tracing::debug!("Found PythonEnd");
                    // End of Python block
                    *in_python_block = false;
                    return Ok(());
                }
                Ok(Token::Comma) => {
                    tracing::debug!("Found Comma");
                    // Skip comma and continue looking for next function
                    continue;
                }
                Ok(Token::Identifier(name)) => {
                    tracing::debug!("Found Identifier: {}", name);
                    // Found a function name, now check for opening parenthesis
                    if let Some(Ok(Token::ParenOpen)) = lexer.next() {
                        tracing::debug!("Found opening parenthesis for {}", name);
                        // Parse the function arguments
                        let func_call = parse_function_with_kwargs(lexer, name)?;
                        tracing::debug!("Parsed function: {:?}", func_call);
                        result.push(func_call);
                        found_function = true;
                        break;
                    }
                }
                _ => continue,
            }
        }

        if !found_function {
            tracing::debug!("No more functions found");
            // If we didn't find a function, we've reached the end of input
            break;
        }

        // After parsing a function, we need to check if there's a comma (more functions)
        // or closing bracket (end of list)
        let mut next_is_comma = false;
        let mut list_ended = false;

        tracing::debug!("Looking for comma or closing bracket...");
        for token in lexer.by_ref() {
            tracing::debug!("Post-func token: {:?}", token);
            match token {
                Ok(Token::BracketClose) => {
                    tracing::debug!("Found closing bracket");
                    // End of the list
                    list_ended = true;
                    break;
                }
                Ok(Token::Comma) => {
                    tracing::debug!("Found comma, more functions to come");
                    // More functions to come
                    next_is_comma = true;
                    break;
                }
                Ok(Token::PythonEnd) => {
                    tracing::debug!("Found PythonEnd");
                    // End of Python block
                    *in_python_block = false;
                    return Ok(());
                }
                _ => {
                    tracing::debug!("Skipping other token: {:?}", token);
                    continue; // Skip any other tokens
                }
            }
        }

        if list_ended || !next_is_comma {
            tracing::debug!(
                "List ended: {}, next_is_comma: {}",
                list_ended,
                next_is_comma
            );
            // If we found closing bracket or didn't find a comma, we're done
            break;
        }
    }

    Ok(())
}

/// Helper function to handle post-value tokens (comma or closing parenthesis)
pub fn handle_post_value(
    lexer: &mut Lexer<'_, Token>,
    name: String,
    kwargs: HashMap<String, Value>,
) -> Result<Value> {
    match lexer.next() {
        Some(Ok(Token::Comma)) => {
            tracing::debug!("handle_post_value: Found comma - continue to next parameter");
            // Continue to next parameter
            Ok(Value::Empty) // Signal to continue
        }
        Some(Ok(Token::ParenClose)) => {
            tracing::debug!(
                "handle_post_value: Found closing parenthesis - end of args for {}",
                name
            );
            // End of arguments
            Ok(Value::FunctionCall(FunctionCall { name, kwargs }))
        }
        other => {
            tracing::debug!("handle_post_value: Unexpected token: {:?}", other);
            // Skip unexpected tokens and continue
            Ok(Value::Empty) // Signal to continue
        }
    }
}

/// Parse a function call with keyword arguments
pub fn parse_function_with_kwargs(lexer: &mut Lexer<'_, Token>, name: String) -> Result<Value> {
    tracing::debug!("Parsing function {} with kwargs", name);
    let mut kwargs = HashMap::new();

    loop {
        match lexer.next() {
            Some(Ok(Token::PythonStart)) => {
                tracing::debug!("Found PythonStart in kwargs");
                // Start of a new Python block
                return Ok(Value::FunctionCall(FunctionCall { name, kwargs }));
            }
            Some(Ok(Token::ParenClose)) => {
                tracing::debug!("Found ParenClose - end of arguments for {}", name);
                // End of arguments
                return Ok(Value::FunctionCall(FunctionCall { name, kwargs }));
            }
            Some(Ok(Token::Identifier(key))) => {
                tracing::debug!("Found parameter key: {}", key);
                // Expect an equals sign
                if let Some(Ok(Token::Equals)) = lexer.next() {
                    tracing::debug!("Found equals sign for {}", key);
                    // Look for value
                    match lexer.next() {
                        Some(Ok(Token::String(val))) => {
                            tracing::debug!("Found string value: {} for {}", val, key);
                            kwargs.insert(key, Value::String(val));
                            let result = handle_post_value(lexer, name.clone(), kwargs.clone())?;
                            if let Value::FunctionCall(_) = result {
                                return Ok(result);
                            }
                        }
                        Some(Ok(Token::Bool(val))) => {
                            tracing::debug!("Found bool value: {} for {}", val, key);
                            kwargs.insert(key, Value::Bool(val));
                            let result = handle_post_value(lexer, name.clone(), kwargs.clone())?;
                            if let Value::FunctionCall(_) = result {
                                return Ok(result);
                            }
                        }
                        Some(Ok(Token::Number(val))) => {
                            tracing::debug!("Found number value: {} for {}", val, key);
                            kwargs.insert(key, Value::Number(val));
                            let result = handle_post_value(lexer, name.clone(), kwargs.clone())?;
                            if let Value::FunctionCall(_) = result {
                                return Ok(result);
                            }
                        }
                        Some(Ok(Token::Identifier(val))) => {
                            tracing::debug!("Found identifier value: {} for {}", val, key);
                            kwargs.insert(key, Value::Identifier(val));
                            let result = handle_post_value(lexer, name.clone(), kwargs.clone())?;
                            if let Value::FunctionCall(_) = result {
                                return Ok(result);
                            }
                        }
                        Some(Ok(Token::Comma)) => {
                            tracing::debug!("Found comma after equals - empty parameter");
                            // Empty parameter value (key=,)
                            kwargs.insert(key, Value::Empty);
                            // Continue to next parameter
                            continue;
                        }
                        Some(Ok(Token::ParenClose)) => {
                            tracing::debug!(
                                "Found ParenClose after equals - empty parameter at end"
                            );
                            // Empty parameter at the end (key=))
                            kwargs.insert(key, Value::Empty);
                            return Ok(Value::FunctionCall(FunctionCall { name, kwargs }));
                        }
                        other => {
                            tracing::debug!("Unexpected token after equals: {:?}", other);
                            // For any other token, treat it as an empty value and continue
                            kwargs.insert(key, Value::Empty);
                            continue;
                        }
                    }
                }
            }
            Some(Ok(Token::Comma)) => {
                tracing::debug!("Found extra comma in arguments");
                // Extra comma, continue
                continue;
            }
            Some(Ok(Token::BracketOpen)) => {
                tracing::debug!("Found BracketOpen in function args - nested list");
                // We've reached a nested list - we're done with this function call
                return Ok(Value::FunctionCall(FunctionCall { name, kwargs }));
            }
            None => {
                tracing::debug!("Reached end of input in function args");
                // End of input
                return Ok(Value::FunctionCall(FunctionCall { name, kwargs }));
            }
            Some(Ok(Token::PythonEnd)) => {
                tracing::debug!("Found PythonEnd in function args");
                // End of Python block
                return Ok(Value::FunctionCall(FunctionCall { name, kwargs }));
            }
            other => {
                tracing::debug!("Skipping other token in function args: {:?}", other);
                // Skip any other tokens
                continue;
            }
        }
    }
}
