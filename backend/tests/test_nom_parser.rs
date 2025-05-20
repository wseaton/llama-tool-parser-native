use backend::{FunctionCall, NomParserState, Value, parse_incremental, parse_python_with_nom};
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

#[test]
fn test_basic_function_call() {
    let input = r#"[test_function(arg1="value1", arg2=42)]"#;

    let expected = vec![create_function_call(
        "test_function",
        vec![
            ("arg1", Value::String("value1".to_string())),
            ("arg2", Value::Number(42.0)),
        ],
    )];

    let result = parse_python_with_nom(input).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_multiple_function_calls() {
    let input = r#"[func1(arg="val1"), func2(arg=42)]"#;

    let expected = vec![
        create_function_call("func1", vec![("arg", Value::String("val1".to_string()))]),
        create_function_call("func2", vec![("arg", Value::Number(42.0))]),
    ];

    let result = parse_python_with_nom(input).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_python_markers() {
    let input = r#"<|python_start|>[test_function(arg="value")]<|python_end|>"#;

    let expected = vec![create_function_call(
        "test_function",
        vec![("arg", Value::String("value".to_string()))],
    )];

    let result = parse_python_with_nom(input).unwrap();
    assert_eq!(result, expected);
}

#[test]
fn test_incremental_parsing() {
    let mut state = NomParserState::new();

    // Send incremental chunks
    let _ = parse_incremental(&mut state, "[test_function(");
    assert_eq!(state.parsed_functions.len(), 0);

    let _ = parse_incremental(&mut state, "arg1=\"value1\", ");
    assert_eq!(state.parsed_functions.len(), 0);

    let result = parse_incremental(&mut state, "arg2=42)]").unwrap();

    let expected = vec![create_function_call(
        "test_function",
        vec![
            ("arg1", Value::String("value1".to_string())),
            ("arg2", Value::Number(42.0)),
        ],
    )];

    assert_eq!(result, expected);
}

#[test]
fn test_incremental_multiple_functions() {
    let mut state = NomParserState::new();

    // First function complete, second partial
    let _ = parse_incremental(&mut state, "[func1(arg=\"val1\"), func2(");
    assert_eq!(state.parsed_functions.len(), 0);

    // Complete the second function
    let result = parse_incremental(&mut state, "arg=42)]").unwrap();

    let expected = vec![
        create_function_call("func1", vec![("arg", Value::String("val1".to_string()))]),
        create_function_call("func2", vec![("arg", Value::Number(42.0))]),
    ];

    assert_eq!(result, expected);
}

#[test]
fn test_boolean_values() {
    let input = r#"[test_function(flag1=True, flag2=False)]"#;

    let expected = vec![create_function_call(
        "test_function",
        vec![("flag1", Value::Bool(true)), ("flag2", Value::Bool(false))],
    )];

    let result = parse_python_with_nom(input).unwrap();
    assert_eq!(result, expected);
}
