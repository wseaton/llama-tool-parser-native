from llama_tool_parser_native import parse_tools
import time


def test_comma_separated_function_calls():
    """Test with comma-separated function calls in a single list."""
    # Simplified test case with just two function calls for easier debugging
    code = "[func1(arg1=\"val1\"), func2(arg2=\"val2\")]"

    print(f"Simplified test code: {code}")
    start_time = time.time()
    tools = parse_tools(code)
    end_time = time.time()
    print(f"Parsing took {(end_time - start_time) * 1000:.2f} milliseconds")
    print(f"Result: {tools}")

    # Assertions
    assert isinstance(tools, list)
    assert len(tools) == 2
    
    # Only if first test passes, try the original complex case
    if len(tools) == 2:
        print("\nNow testing full example:")
        complex_code = """
        [get_weather_forecast(location="Tokyo", days=7), search_hotels(location="Shinjuku", check_in_date="2024-05-20", check_out_date="2024-05-27", budget_max_per_night=50.0, guest_count=2), get_attractions(location="Tokyo", count=3, category="all"), convert_currency(amount=1000, from_currency="USD", to_currency="JPY")]
        """
        print(f"Complex code: {complex_code}")
        complex_tools = parse_tools(complex_code)
        print(f"Complex result: {complex_tools}")
        assert len(complex_tools) == 4
        
        # Check for specific tool names
        tool_names = [tool["name"] for tool in complex_tools]
        assert "get_weather_forecast" in tool_names
        assert "search_hotels" in tool_names
        assert "get_attractions" in tool_names
        assert "convert_currency" in tool_names