from llama_tool_parser_native import parse_tools
import time


def test_simple_function_calls():
    """Test with various simple function calls in llama format."""
    code = """
    [get_weather(location="San Francisco", unit="celsius")]
    [add_calendar_event(title="Team Meeting", start_time="2024-05-20T14:00:00Z", duration_minutes=60)]
    [search_web(query="rust programming language", max_results=10)]
    [send_email(to="user@example.com", subject="Meeting Summary", body="Here are the action items from our meeting.")]
    [calculate(expression="5 * (3 + 2)", format="decimal")]
    """

    start_time = time.time()
    tools = parse_tools(code)
    end_time = time.time()
    print(f"Parsing took {(end_time - start_time) * 1000:.2f} milliseconds")
    print(tools)

    # Assertions
    assert isinstance(tools, list)
    assert len(tools) == 5

    # Check contents of first tool
    assert tools[0]["name"] == "get_weather"
    assert tools[0]["kwargs"]["location"] == {"String": "San Francisco"}
    assert tools[0]["kwargs"]["unit"] == {"String": "celsius"}

    # Check contents of third tool
    assert tools[2]["name"] == "search_web"
    assert tools[2]["kwargs"]["query"] == {"String": "rust programming language"}
    assert tools[2]["kwargs"]["max_results"] == {"Number": 10.0}


def test_nested_function_calls():
    """Test with nested function calls and complex structures."""
    code = """Some text before
    <|python_start|>
    [execute_commands(commands=[
        [system_command(command="ls -la", timeout=30)],
        [database_query(query="SELECT * FROM users", limit=100)],
        [api_request(endpoint="/status", method="GET")]
    ])]
    <|python_end|>
    Some text after
    <|python_start|>
    [format_data(data=[
        [create_record(name="John", age=30, active=True)],
        [create_record(name="Alice", age=25, active=False)]
    ], output_format="json")]
    <|python_end|>
    """

    start_time = time.time()
    tools = parse_tools(code)
    end_time = time.time()
    print(f"Parsing took {(end_time - start_time) * 1000:.2f} milliseconds")
    print(tools)

    # Assertions
    assert isinstance(tools, list)
    # The function should extract individual function calls
    assert len(tools) > 2


def test_edge_cases():
    """Test various edge cases for the parser."""
    code = """
    # Empty parameter values
    [configure_settings(theme=, language=, notifications=True)]
    
    # Mixed parameter types
    [update_user(id=12345, name="John Doe", active=True, metadata=, roles=["admin", "user"])]
    
    # Unusual formatting with line breaks
    [send_message(
        recipient="user@example.com",
        message="Hello
        World",
        priority="high"
    )]
    
    # Multiple sequential brackets
    [[analyze_data(source="sensors", timeframe="1h")]]
    
    # Malformed calls that should still be parsed if possible
    [incomplete_function(param1="value1"
    [missing_closing_bracket(param="test")]
    [missing_closing_paren(param="test"]
    """

    start_time = time.time()
    tools = parse_tools(code)
    end_time = time.time()
    print(f"Parsing took {(end_time - start_time) * 1000:.2f} milliseconds")
    print(tools)

    # Basic assertion - we should get at least some results
    assert isinstance(tools, list)


# Advanced test case with real-world-like formatting
def test_realistic_llm_output():
    """Test with formatting similar to actual LLM outputs."""
    code = """
    I'll help you with that task. Let me use some tools to accomplish this:
    
    [search_documentation(query="rust parser combinators", sections=["tutorials", "api"])]
    
    Based on the search results, I recommend using the following approach:
    
    [execute_code(
        language="rust",
        code="
            fn main() {
                println!(\"Hello, world!\");
            }
        ",
        save_to="example.rs"
    )]
    
    Now I'll analyze the performance:
    
    [run_benchmark(
        test_cases=[
            [create_test(name="small_input", size=100)],
            [create_test(name="medium_input", size=1000)],
            [create_test(name="large_input", size=10000)]
        ],
        iterations=5,
        output_format="csv"
    )]
    
    Here are the results of my analysis...
    """

    start_time = time.time()
    tools = parse_tools(code)
    end_time = time.time()
    print(f"Parsing took {(end_time - start_time) * 1000:.2f} milliseconds")
    print(tools)

    # Assertions
    assert isinstance(tools, list)
    # We should have at least the main function calls
    assert len(tools) >= 3

    # Check for specific tool names
    tool_names = [tool["name"] for tool in tools]
    assert "search_documentation" in tool_names
    assert "execute_code" in tool_names
    assert "run_benchmark" in tool_names
