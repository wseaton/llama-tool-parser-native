import json
import unittest
from unittest.mock import MagicMock, patch

from llama_tool_parser_native.pythonic_parser import NativePythonicToolParser
from vllm.entrypoints.openai.protocol import ChatCompletionRequest


class TestNativePythonicToolParser(unittest.TestCase):
    def setUp(self):
        # Mock the tokenizer as it's required by the parser
        mock_tokenizer = MagicMock()
        self.parser = NativePythonicToolParser(tokenizer=mock_tokenizer)

    def test_process_tool_arguments_simple_values(self):
        # Test with simple values (strings, integers, booleans)
        test_kwargs = {
            "name": "John Doe",
            "age": 30,
            "is_active": True,
            "tags": ["user", "premium"],
        }

        result = self.parser._process_tool_arguments(test_kwargs)

        # The function should return the same dictionary for simple values
        self.assertEqual(result, test_kwargs)

    def test_process_tool_arguments_with_single_value_dicts(self):
        # Test with dictionary that contains single-value dictionaries
        test_kwargs = {
            "user": {"name": "John Doe"},  # Should extract "John Doe"
            "settings": {"theme": "dark"},  # Should extract "dark"
            "address": {
                "street": "123 Main St",
                "city": "New York",
            },  # Should not change this dict
            "simple_value": "hello",  # Should not change simple values
        }

        expected = {
            "user": "John Doe",  # Extracted from single-value dict
            "settings": "dark",  # Extracted from single-value dict
            "address": {"street": "123 Main St", "city": "New York"},  # Unchanged
            "simple_value": "hello",  # Unchanged
        }

        result = self.parser._process_tool_arguments(test_kwargs)

        # The function should extract values from single-value dictionaries
        self.assertEqual(result, expected)

    def test_process_tool_arguments_with_nested_structure(self):
        # Test with more complex nested structure
        test_kwargs = {
            "user": {
                "profile": {
                    "name": "John"
                }  # Nested single-value dict, extracted from profile
            },
            "metadata": {
                "timestamp": 1621234567
            },  # Single-value dict, should extract value
        }

        expected = {
            "user": {"name": "John"},  # Extracted from user.profile
            "metadata": 1621234567,  # Extracted value
        }

        result = self.parser._process_tool_arguments(test_kwargs)

        self.assertEqual(result, expected)

    @patch("llama_tool_parser_native.pythonic_parser.parse_tools")
    def test_extract_tool_calls_with_no_tools(self, mock_parse_tools):
        # Mock parse_tools to return empty list (no tool calls found)
        mock_parse_tools.return_value = []

        # Create a mock request
        mock_request = MagicMock(spec=ChatCompletionRequest)

        # Call the function
        result = self.parser.extract_tool_calls("some model output", mock_request)

        # Verify the result is as expected
        self.assertFalse(result.tools_called)
        self.assertEqual(result.tool_calls, [])
        self.assertEqual(result.content, "some model output")

    @patch("llama_tool_parser_native.pythonic_parser.parse_tools")
    def test_extract_tool_calls_with_tools(self, mock_parse_tools):
        # Mock the actual output format from parse_tools based on the Rust implementation
        # parse_tools returns a list of FunctionCall objects with name and kwargs fields
        mock_parse_tools.return_value = [
            {
                "name": "search",
                "kwargs": {
                    "query": "weather forecast",
                    "options": {
                        "limit": 5
                    },  # Single-value dict that should be simplified
                },
            },
            {"name": "calculate", "kwargs": {"expression": "2 + 2", "precision": 2}},
        ]

        # Create a mock request
        mock_request = MagicMock(spec=ChatCompletionRequest)

        # Call the function
        result = self.parser.extract_tool_calls(
            "some model output with tools", mock_request
        )

        # Verify the result is as expected
        self.assertTrue(result.tools_called)
        self.assertEqual(len(result.tool_calls), 2)

        # Check the first tool call
        self.assertEqual(result.tool_calls[0].type, "function")
        self.assertEqual(result.tool_calls[0].function.name, "search")

        # Parse the arguments JSON and verify content
        search_args = json.loads(result.tool_calls[0].function.arguments)
        self.assertEqual(search_args["query"], "weather forecast")
        self.assertEqual(
            search_args["options"], 5
        )  # Should be simplified from {"limit": 5}

        # Check the second tool call
        self.assertEqual(result.tool_calls[1].type, "function")
        self.assertEqual(result.tool_calls[1].function.name, "calculate")

        # Parse the arguments JSON and verify content
        calc_args = json.loads(result.tool_calls[1].function.arguments)
        self.assertEqual(calc_args["expression"], "2 + 2")
        self.assertEqual(calc_args["precision"], 2)

        # The original model output should be preserved
        self.assertEqual(result.content, "some model output with tools")

    def test_e2e_extract_tool_calls(self):
        """
        End-to-end test with a realistic model output and real parser.
        NOTE: This test requires the actual native parser to be built and available.
        """
        # Create a real (not mocked) parser instance
        parser = NativePythonicToolParser(tokenizer=MagicMock())

        # Create a mock request
        mock_request = MagicMock(spec=ChatCompletionRequest)

        # Model output with pythonic tool calls
        model_output = """I'll search for information about the weather forecast.

<|python_start|>
[search(query="weather forecast San Francisco", num_results=3)]
<|python_end|>

Here are the weather results for San Francisco."""

        # Process the output
        result = parser.extract_tool_calls(model_output, mock_request)

        # Verify the result contains the tool call
        self.assertTrue(result.tools_called)
        self.assertEqual(len(result.tool_calls), 1)

        # Verify the tool call details
        tool_call = result.tool_calls[0]
        self.assertEqual(tool_call.type, "function")
        self.assertEqual(tool_call.function.name, "search")

        # Parse the arguments and verify
        args = json.loads(tool_call.function.arguments)
        self.assertEqual(args["query"], "weather forecast San Francisco")
        self.assertEqual(args["num_results"], 3)


if __name__ == "__main__":
    unittest.main()
