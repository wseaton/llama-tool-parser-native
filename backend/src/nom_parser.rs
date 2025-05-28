use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{escaped, tag, take_till, take_until, take_while},
    character::complete::{char, digit1, multispace0, one_of},
    combinator::{map, map_res, opt, recognize, value},
    multi::{many0, many1, separated_list0},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
};
use std::collections::HashMap;
use std::str::FromStr;

use crate::{FunctionCall, Value};

// Parser state for incremental parsing
#[derive(Debug, Clone)]
pub struct NomParserState {
    // Any partial data from previous parse attempts
    pub remainder: String,
    // Functions we've already successfully parsed
    pub parsed_functions: Vec<FunctionCall>,
    // Are we currently inside a Python block
    pub in_python_block: bool,
    // Are we inside a function list
    pub in_function_list: bool,
    // Current function being built
    pub current_function: Option<PartialFunction>,
}

// Track a function being parsed
#[derive(Debug, Clone)]
pub struct PartialFunction {
    pub name: String,
    pub kwargs: HashMap<String, Value>,
    // inside the function's parentheses?
    pub in_args: bool,
}

impl NomParserState {
    pub fn new() -> Self {
        Self {
            remainder: String::new(),
            parsed_functions: Vec::new(),
            in_python_block: false,
            in_function_list: false,
            current_function: None,
        }
    }

    pub fn reset(&mut self) {
        self.remainder = String::new();
        self.parsed_functions = Vec::new();
        self.in_python_block = false;
        self.in_function_list = false;
        self.current_function = None;
    }

    pub fn add_input(&mut self, input: &str) {
        self.remainder.push_str(input);
    }

    pub fn get_parsed_functions(&self) -> Vec<FunctionCall> {
        self.parsed_functions.clone()
    }
}

impl Default for NomParserState {
    fn default() -> Self {
        Self::new()
    }
}

// Parse a boolean
fn parse_bool(input: &str) -> IResult<&str, bool> {
    alt((value(true, tag("True")), value(false, tag("False"))))(input)
}

// Helper function to handle escaped characters
fn unescape_string(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('\\') => result.push('\\'),
                Some('\"') => result.push('\"'),
                Some('\'') => result.push('\''),
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some(other) => {
                    // For any other escaped character, just keep it
                    result.push(other);
                }
                None => {
                    // Handle case where backslash is at the end
                    result.push('\\');
                }
            }
        } else {
            result.push(c);
        }
    }
    
    result
}

// Parse a string with escape sequences (either single or double quoted)
fn parse_string(input: &str) -> IResult<&str, String> {
    alt((
        // Double quoted string - more permissive with escaped characters
        map(
            delimited(
                char('"'),
                escaped(
                    take_while(|c| c != '"' && c != '\\'),
                    '\\',
                    one_of("\"\\nrt!(){}[].;:"), // Accept common escaped characters
                ),
                char('"'),
            ),
            unescape_string,
        ),
        // Single quoted string - more permissive with escaped characters
        map(
            delimited(
                char('\''),
                escaped(
                    take_while(|c| c != '\'' && c != '\\'),
                    '\\',
                    one_of("'\\nrt!(){}[].;:"), // Accept common escaped characters
                ),
                char('\''),
            ),
            unescape_string,
        ),
    ))(input)
}

// Parse a number (integer or float)
fn parse_number(input: &str) -> IResult<&str, f64> {
    map_res(
        recognize(tuple((
            opt(char('-')),
            digit1,
            opt(tuple((char('.'), digit1))),
            opt(tuple((one_of("eE"), opt(one_of("+-")), digit1))),
        ))),
        |s: &str| f64::from_str(s),
    )(input)
}

// Parse an identifier
fn parse_identifier(input: &str) -> IResult<&str, String> {
    map(
        recognize(pair(
            one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_"),
            take_while(|c: char| c.is_alphanumeric() || c == '_'),
        )),
        |s: &str| s.to_string(),
    )(input)
}

// Forward declaration to handle recursive types
fn parse_value(input: &str) -> IResult<&str, Value> {
    preceded(
        multispace0,
        alt((
            map(parse_bool, Value::Bool),
            map(parse_string, Value::String),
            map(parse_number, Value::Number),
            map(tag("None"), |_| Value::Empty),
            parse_list,
            parse_dict,
            map(parse_identifier, Value::Identifier),
        )),
    )(input)
}

// Parse a list: [value1, value2, ...]
fn parse_list(input: &str) -> IResult<&str, Value> {
    map(
        delimited(
            char('['),
            separated_list0(
                preceded(multispace0, char(',')),
                preceded(multispace0, parse_value),
            ),
            preceded(multispace0, char(']')),
        ),
        Value::List,
    )(input)
}

// Parse a dict: {'key1': value1, 'key2': value2, ...}
fn parse_dict(input: &str) -> IResult<&str, Value> {
    // Parse a dict directly
    delimited(
        char('{'),
        map(
            separated_list0(
                preceded(multispace0, char(',')),
                preceded(
                    multispace0,
                    separated_pair(
                        // Keys must be strings
                        parse_string, 
                        preceded(multispace0, char(':')), 
                        parse_value
                    ),
                ),
            ),
            |entries| {
                // Convert the entries to a list with alternating keys and values
                let mut values = Vec::new();
                for (key, value) in entries {
                    values.push(Value::String(key));
                    values.push(value);
                }
                Value::List(values)
            },
        ),
        preceded(multispace0, char('}')),
    )(input)
}

// Parse a keyword argument
fn parse_kwarg(input: &str) -> IResult<&str, (String, Value)> {
    separated_pair(
        parse_identifier,
        preceded(multispace0, char('=')),
        preceded(multispace0, parse_value),
    )(input)
}

// Parse a function's arguments
fn parse_kwargs(input: &str) -> IResult<&str, HashMap<String, Value>> {
    map(
        delimited(
            char('('),
            separated_list0(
                preceded(multispace0, char(',')),
                preceded(multispace0, parse_kwarg),
            ),
            preceded(multispace0, char(')')),
        ),
        |pairs| pairs.into_iter().collect(),
    )(input)
}

// Parse a function call: name(arg1="value1", arg2=42)
fn parse_function_call(input: &str) -> IResult<&str, FunctionCall> {
    map(pair(parse_identifier, parse_kwargs), |(name, kwargs)| {
        FunctionCall { name, kwargs }
    })(input)
}

// Parse a list of function calls: [func1(arg1="val1"), func2(arg2="val2")]
fn parse_function_list(input: &str) -> IResult<&str, Vec<FunctionCall>> {
    delimited(
        char('['),
        separated_list0(
            preceded(multispace0, char(',')),
            preceded(multispace0, parse_function_call),
        ),
        preceded(multispace0, char(']')),
    )(input)
}

// Parse a Python block: <|python_start|>[function_calls]<|python_end|>
fn parse_python_block(input: &str) -> IResult<&str, Vec<FunctionCall>> {
    delimited(
        tag("<|python_start|>"),
        parse_function_list,
        tag("<|python_end|>"),
    )(input)
}

// Top-level parser that handles both Python blocks and bare function lists
pub fn parse_python_nom(input: &str) -> IResult<&str, Vec<FunctionCall>> {
    alt((parse_python_block, parse_function_list))(input)
}

// Parse function calls that may be anywhere in the text with surrounding content
pub fn parse_python_with_surrounding_text(input: &str) -> Result<Vec<FunctionCall>, String> {
    let mut all_functions = Vec::new();
    let mut remaining = input;
    
    // Continue searching through the text until we've processed it all
    while !remaining.is_empty() {
        // Try to find a Python block or function list starting anywhere in the remaining text
        if let Some(start_pos) = find_next_pattern_start(remaining) {
            // Skip to the start of the pattern
            let from_pattern = &remaining[start_pos..];
            
            // Try to parse from this position
            match parse_python_nom(from_pattern) {
                Ok((rest, mut functions)) => {
                    // Add the found functions
                    all_functions.append(&mut functions);
                    // Continue with the remaining text after this parse
                    remaining = rest;
                }
                Err(_) => {
                    // If parsing failed, skip this character and try again
                    if remaining.len() > start_pos + 1 {
                        remaining = &remaining[start_pos + 1..];
                    } else {
                        break;
                    }
                }
            }
        } else {
            // No more patterns found
            break;
        }
    }
    
    Ok(all_functions)
}

// Find the next position where a Python block or function list might start
fn find_next_pattern_start(input: &str) -> Option<usize> {
    // Look for either "<|python_start|>" or "["
    let python_start = input.find("<|python_start|>");
    let bracket_start = input.find('[');
    
    match (python_start, bracket_start) {
        (Some(p), Some(b)) => Some(p.min(b)),
        (Some(p), None) => Some(p),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

// Parse a string and return function calls, similar to the original parser
pub fn parse_python_with_nom(source: &str) -> Result<Vec<FunctionCall>, String> {
    // First try the new approach that handles surrounding text
    match parse_python_with_surrounding_text(source) {
        Ok(functions) if !functions.is_empty() => Ok(functions),
        _ => {
            // Fall back to the strict parser for backwards compatibility
            match parse_python_nom(source) {
                Ok((_, function_calls)) => Ok(function_calls),
                Err(e) => Err(format!("Parse error: {:?}", e)),
            }
        }
    }
}

// Incremental parsing function that maintains state
pub fn parse_incremental(
    state: &mut NomParserState,
    chunk: &str,
) -> Result<Vec<FunctionCall>, String> {
    // Add new chunk to existing remainder
    state.add_input(chunk);
    let input = &state.remainder;

    // Use the new surrounding text parser for better compatibility
    match parse_python_with_surrounding_text(input) {
        Ok(function_calls) => {
            // For incremental parsing, we need to be more careful about what's complete
            // Check if we have complete function calls by trying the strict parser on parts
            let mut new_functions = Vec::new();
            
            // Try to find complete patterns and parse them
            for func in function_calls {
                // Only add functions that weren't already parsed
                if !state.parsed_functions.iter().any(|existing| 
                    existing.name == func.name && existing.kwargs == func.kwargs) {
                    new_functions.push(func);
                }
            }
            
            // Add new functions to our state
            state.parsed_functions.extend(new_functions);
            
            // For streaming, we might want to clear some of the remainder to avoid reprocessing
            // but for now, let's keep it simple
            Ok(state.parsed_functions.clone())
        }
        Err(e) => {
            // If the new parser fails, fall back to the old approach
            tracing::debug!("Incremental parse error with surrounding text parser: {:?}", e);
            // Try the strict parser as fallback
            match parse_python_nom(input) {
                Ok((remainder, mut function_calls)) => {
                    state.remainder = remainder.to_string();
                    state.parsed_functions.append(&mut function_calls);
                    Ok(state.parsed_functions.clone())
                }
                Err(nom::Err::Incomplete(_)) => {
                    // Not enough data yet, keep accumulating
                    Ok(state.parsed_functions.clone())
                }
                Err(_) => {
                    // Return what we have so far
                    Ok(state.parsed_functions.clone())
                }
            }
        }
    }
}
