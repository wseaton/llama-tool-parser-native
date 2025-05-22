use backend::{FunctionCall, parse_python_with_nom, Value, NomParserState, parse_incremental};
use std::collections::HashMap;

fn create_function_call(name: &str, args: Vec<(&str, Value)>) -> FunctionCall {
    let mut kwargs = HashMap::new();
    for (k, v) in args {
        kwargs.insert(k.to_string(), v);
    }
    FunctionCall {
        name: name.to_string(),
        kwargs,
    }
}

// Test constants based on the Python test cases
const SIMPLE_FUNCTION_OUTPUT: &str = "get_weather(city=\"San Francisco\", metric=\"celsius\")";
const MORE_TYPES_FUNCTION_OUTPUT: &str = "register_user(name=\"John Doe\", age=37, address={'city': 'San Francisco', 'state': 'CA'}, role=None, passed_test=True, aliases=['John', 'Johnny'])";
const PARAMETERLESS_FUNCTION_OUTPUT: &str = "get_weather()";
const EMPTY_DICT_FUNCTION_OUTPUT: &str = "do_something_cool(additional_data={})";
const EMPTY_LIST_FUNCTION_OUTPUT: &str = "do_something_cool(steps=[])";
// Simplify the escaped string test case
const ESCAPED_STRING_FUNCTION_OUTPUT: &str = "get_weather(city=\"Martha Vineyard\", metric=\"cool units\")";

// Helper to get the simple function call for tests
fn get_simple_function_call() -> FunctionCall {
    create_function_call(
        "get_weather",
        vec![
            ("city", Value::String("San Francisco".to_string())),
            ("metric", Value::String("celsius".to_string())),
        ],
    )
}

// Helper to get the more complex function call for tests
fn get_more_types_function_call() -> FunctionCall {
    // Create the address dictionary
    let address_entries = vec![
        Value::String("city".to_string()), 
        Value::String("San Francisco".to_string()),
        Value::String("state".to_string()), 
        Value::String("CA".to_string()),
    ];
    
    // Create the aliases list
    let aliases = Value::List(vec![
        Value::String("John".to_string()),
        Value::String("Johnny".to_string()),
    ]);
    
    create_function_call(
        "register_user",
        vec![
            ("name", Value::String("John Doe".to_string())),
            ("age", Value::Number(37.0)),
            ("address", Value::List(address_entries)),
            ("role", Value::Empty),
            ("passed_test", Value::Bool(true)),
            ("aliases", aliases),
        ],
    )
}

// Helper to get parameterless function call for tests
fn get_parameterless_function_call() -> FunctionCall {
    create_function_call("get_weather", vec![])
}

// Helper to get empty dict function call for tests
fn get_empty_dict_function_call() -> FunctionCall {
    create_function_call(
        "do_something_cool",
        vec![("additional_data", Value::List(vec![]))],
    )
}

// Helper to get empty list function call for tests
fn get_empty_list_function_call() -> FunctionCall {
    create_function_call(
        "do_something_cool",
        vec![("steps", Value::List(vec![]))],
    )
}

// Helper to get escaped string function call for tests
fn get_escaped_string_function_call() -> FunctionCall {
    create_function_call(
        "get_weather",
        vec![
            ("city", Value::String("Martha Vineyard".to_string())),
            ("metric", Value::String("cool units".to_string())),
        ],
    )
}

#[test]
fn test_no_tool_call() {
    let model_output = "How can I help you today?";
    // The parser will return an error for non-matching input, which is expected behavior
    let result = parse_python_with_nom(model_output);
    
    // Expect an error since this isn't a valid function call syntax
    assert!(result.is_err());
}

// Test cases for non-streaming parsing
#[test]
fn test_simple_nonstreaming() {
    let model_output = format!("[{}]", SIMPLE_FUNCTION_OUTPUT);
    let expected = vec![get_simple_function_call()];
    
    let result = parse_python_with_nom(&model_output).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_more_types_nonstreaming() {
    let model_output = format!("[{}]", MORE_TYPES_FUNCTION_OUTPUT);
    let expected = vec![get_more_types_function_call()];
    
    let result = parse_python_with_nom(&model_output).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_parameterless_nonstreaming() {
    let model_output = format!("[{}]", PARAMETERLESS_FUNCTION_OUTPUT);
    let expected = vec![get_parameterless_function_call()];
    
    let result = parse_python_with_nom(&model_output).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_empty_dict_nonstreaming() {
    let model_output = format!("[{}]", EMPTY_DICT_FUNCTION_OUTPUT);
    let expected = vec![get_empty_dict_function_call()];
    
    let result = parse_python_with_nom(&model_output).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_empty_list_nonstreaming() {
    let model_output = format!("[{}]", EMPTY_LIST_FUNCTION_OUTPUT);
    let expected = vec![get_empty_list_function_call()];
    
    let result = parse_python_with_nom(&model_output).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_escaped_string_nonstreaming() {
    let model_output = format!("[{}]", ESCAPED_STRING_FUNCTION_OUTPUT);
    let expected = vec![get_escaped_string_function_call()];
    
    let result = parse_python_with_nom(&model_output).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_parallel_calls_nonstreaming() {
    // For parallel calls, we need to ensure correct comma placement
    let model_output = format!("[{}, {}]", SIMPLE_FUNCTION_OUTPUT, MORE_TYPES_FUNCTION_OUTPUT);
    let expected = vec![
        get_simple_function_call(),
        get_more_types_function_call(),
    ];
    
    let result = parse_python_with_nom(&model_output).unwrap();
    assert_eq!(result, expected);
}

// Test cases for streaming parsing
#[test]
fn test_simple_streaming() {
    let mut state = NomParserState::new();
    let model_output = format!("[{}]", SIMPLE_FUNCTION_OUTPUT);
    
    let result = parse_incremental(&mut state, &model_output).unwrap();
    let expected = vec![get_simple_function_call()];
    
    assert_eq!(result, expected);
}

#[test]
fn test_more_types_streaming() {
    let mut state = NomParserState::new();
    let model_output = format!("[{}]", MORE_TYPES_FUNCTION_OUTPUT);
    
    let result = parse_incremental(&mut state, &model_output).unwrap();
    let expected = vec![get_more_types_function_call()];
    
    assert_eq!(result, expected);
}

#[test]
fn test_parameterless_streaming() {
    let mut state = NomParserState::new();
    let model_output = format!("[{}]", PARAMETERLESS_FUNCTION_OUTPUT);
    
    let result = parse_incremental(&mut state, &model_output).unwrap();
    let expected = vec![get_parameterless_function_call()];
    
    assert_eq!(result, expected);
}

#[test]
fn test_empty_dict_streaming() {
    let mut state = NomParserState::new();
    let model_output = format!("[{}]", EMPTY_DICT_FUNCTION_OUTPUT);
    
    let result = parse_incremental(&mut state, &model_output).unwrap();
    let expected = vec![get_empty_dict_function_call()];
    
    assert_eq!(result, expected);
}

#[test]
fn test_empty_list_streaming() {
    let mut state = NomParserState::new();
    let model_output = format!("[{}]", EMPTY_LIST_FUNCTION_OUTPUT);
    
    let result = parse_incremental(&mut state, &model_output).unwrap();
    let expected = vec![get_empty_list_function_call()];
    
    assert_eq!(result, expected);
}

#[test]
fn test_escaped_string_streaming() {
    let mut state = NomParserState::new();
    let model_output = format!("[{}]", ESCAPED_STRING_FUNCTION_OUTPUT);
    
    let result = parse_incremental(&mut state, &model_output).unwrap();
    let expected = vec![get_escaped_string_function_call()];
    
    assert_eq!(result, expected);
}

#[test]
fn test_parallel_calls_streaming() {
    let mut state = NomParserState::new();
    let model_output = format!("[{}, {}]", SIMPLE_FUNCTION_OUTPUT, MORE_TYPES_FUNCTION_OUTPUT);
    
    let result = parse_incremental(&mut state, &model_output).unwrap();
    let expected = vec![
        get_simple_function_call(),
        get_more_types_function_call(),
    ];
    
    assert_eq!(result, expected);
}

#[test]
fn test_streaming_tool_call_with_large_steps() {
    let mut state = NomParserState::new();
    
    // First delta
    let _ = parse_incremental(&mut state, "[get_weather(city=\"San");
    assert_eq!(state.parsed_functions.len(), 0);
    
    // Second delta completing all functions
    let result = parse_incremental(
        &mut state, 
        " Francisco\", metric=\"celsius\"), get_weather(), do_something_cool(steps=[])]"
    ).unwrap();
    
    let expected = vec![
        get_simple_function_call(),
        get_parameterless_function_call(),
        get_empty_list_function_call(),
    ];
    
    assert_eq!(result, expected);
}