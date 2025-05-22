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
        // Double quoted string
        map(
            delimited(
                char('"'),
                escaped(
                    take_while(|c| c != '"' && c != '\\'),
                    '\\',
                    one_of("\"\\nrt"),
                ),
                char('"'),
            ),
            unescape_string,
        ),
        // Single quoted string
        map(
            delimited(
                char('\''),
                escaped(
                    take_while(|c| c != '\'' && c != '\\'),
                    '\\',
                    one_of("'\\nrt"),
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

// Parse a string and return function calls, similar to the original parser
pub fn parse_python_with_nom(source: &str) -> Result<Vec<FunctionCall>, String> {
    match parse_python_nom(source) {
        Ok((_, function_calls)) => Ok(function_calls),
        Err(e) => Err(format!("Parse error: {:?}", e)),
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

    match parse_python_nom(input) {
        Ok((remainder, mut function_calls)) => {
            // Update the state with new data
            state.remainder = remainder.to_string();
            state.parsed_functions.append(&mut function_calls);

            // Return the complete list of parsed functions so far
            Ok(state.parsed_functions.clone())
        }
        Err(nom::Err::Incomplete(_)) => {
            // Not enough data yet, keep accumulating
            Ok(state.parsed_functions.clone())
        }
        Err(e) => {
            // Try to parse as much as possible
            tracing::debug!("Incremental parse error: {:?}", e);

            // Check if we can find any complete function calls
            if let Ok((rem, functions)) = parse_function_list(input) {
                state.remainder = rem.to_string();
                state.parsed_functions.extend(functions);
            }

            // Return what we have so far
            Ok(state.parsed_functions.clone())
        }
    }
}
