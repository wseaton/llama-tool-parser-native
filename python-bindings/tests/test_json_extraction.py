from llama_tool_parser_native import parse_tools
import json
import pytest


@pytest.mark.parametrize("engine", ["nom", "logos"])
def test_multiple_tool_calls_json_extraction(engine):
    """Test extraction of multiple tool calls from a JSON output."""
    # The JSON from the user's example
    json_data = {
        "id": "chatcmpl-e1e62c1a34eb4808a86c670d80c3f15c",
        "object": "chat.completion",
        "created": 1747678354,
        "model": "meta-llama/Llama-3.2-3B-Instruct",
        "choices": [
            {
                "index": 0,
                "message": {
                    "role": "assistant",
                    "reasoning_content": None,
                    "content": '[get_weather_forecast(location="Tokyo", days=7), search_hotels(location="Shinjuku", check_in_date="2024-05-20", check_out_date="2024-05-27", budget_max_per_night=50.0, guest_count=2), get_attractions(location="Tokyo", count=3, category="all"), convert_currency(amount=1000, from_currency="USD", to_currency="JPY")]',
                    "tool_calls": [
                        {
                            "id": "chatcmpl-tool-9ad74b671345406f80c09b604d9063a8",
                            "type": "function",
                            "function": {
                                "name": "get_weather_forecast",
                                "arguments": '{"location": "Tokyo", "days": 7.0}',
                            },
                        }
                    ],
                },
                "logprobs": None,
                "finish_reason": "tool_calls",
                "stop_reason": 128008,
            }
        ],
        "usage": {
            "prompt_tokens": 854,
            "total_tokens": 951,
            "completion_tokens": 97,
            "prompt_tokens_details": None,
        },
        "prompt_logprobs": None,
        "kv_transfer_params": None,
    }

    # Extract the content part that contains the function calls
    content = json_data["choices"][0]["message"]["content"]
    print(f"Content to parse: {content}")

    # Run the parser
    tools = parse_tools(content, engine=engine)
    print(f"Extracted tools: {tools}")

    # Assertions
    assert isinstance(tools, list)
    assert len(tools) == 4  # Should extract all 4 tool calls

    # Check for specific tool names
    tool_names = [tool["name"] for tool in tools]
    assert "get_weather_forecast" in tool_names
    assert "search_hotels" in tool_names
    assert "get_attractions" in tool_names
    assert "convert_currency" in tool_names

    # Check some parameters (values are wrapped in type objects)
    weather_tool = next(
        tool for tool in tools if tool["name"] == "get_weather_forecast"
    )
    assert weather_tool["kwargs"]["location"]["String"] == "Tokyo"
    assert weather_tool["kwargs"]["days"]["Number"] == 7.0

    hotels_tool = next(tool for tool in tools if tool["name"] == "search_hotels")
    assert hotels_tool["kwargs"]["location"]["String"] == "Shinjuku"
    assert hotels_tool["kwargs"]["budget_max_per_night"]["Number"] == 50.0
