#!/usr/bin/env python3
"""Test streaming functionality for the nom parser."""

from llama_tool_parser_native import IncrementalParser

def test_incremental_parser():
    """Test the incremental parser with chunks."""
    parser = IncrementalParser()
    
    # test partial parsing with chunks
    chunk1 = "<|python_start|>["
    chunk2 = 'search(query="test", limit=5'
    chunk3 = "), write_file("
    chunk4 = 'filename="test.txt", content="hello"'
    chunk5 = ")]<|python_end|>"
    
    print("Testing incremental parsing...")
    
    result1 = parser.parse_chunk(chunk1)
    print(f"After chunk 1: {result1}")
    
    result2 = parser.parse_chunk(chunk2)
    print(f"After chunk 2: {result2}")
    
    result3 = parser.parse_chunk(chunk3)
    print(f"After chunk 3: {result3}")
    
    result4 = parser.parse_chunk(chunk4)
    print(f"After chunk 4: {result4}")
    
    result5 = parser.parse_chunk(chunk5)
    print(f"After chunk 5: {result5}")
    
    print("Final parsed functions:", parser.get_parsed_functions())
    
    # assertions to verify the incremental parser works correctly
    assert result1 == []
    assert result2 == []
    assert result3 == []
    assert result4 == []
    assert len(result5) == 2
    
    final_functions = parser.get_parsed_functions()
    assert len(final_functions) == 2
    assert final_functions[0]["name"] == "search"
    assert final_functions[0]["kwargs"]["query"]["String"] == "test"
    assert final_functions[0]["kwargs"]["limit"]["Number"] == 5.0
    assert final_functions[1]["name"] == "write_file"
    assert final_functions[1]["kwargs"]["filename"]["String"] == "test.txt"
    assert final_functions[1]["kwargs"]["content"]["String"] == "hello"

def test_streaming_parser():
    """Test streaming with the pythonic parser."""
    from pythonic_parser import NativePythonicToolParser
    from transformers import AutoTokenizer
    from vllm.entrypoints.openai.protocol import ChatCompletionRequest
    
    # mock tokenizer
    tokenizer = AutoTokenizer.from_pretrained("microsoft/DialoGPT-medium")
    
    parser = NativePythonicToolParser(tokenizer)
    request = ChatCompletionRequest(messages=[], model="test")
    
    # simulate streaming chunks
    chunks = [
        "<|python_start|>[",
        'search(query="test", limit=5',
        "), write_file(",
        'filename="test.txt", content="hello"',
        ")]<|python_end|>"
    ]
    
    previous_text = ""
    current_text = ""
    
    print("\nTesting streaming parser...")
    
    for i, chunk in enumerate(chunks):
        previous_text = current_text
        current_text = previous_text + chunk
        
        result = parser.extract_tool_calls_streaming(
            previous_text=previous_text,
            current_text=current_text,
            delta_text=chunk,
            previous_token_ids=[],
            current_token_ids=[],
            delta_token_ids=[],
            request=request
        )
        
        print(f"Chunk {i+1} ({chunk[:20]}...): {result}")
    
    # assertions to verify the streaming parser works correctly
    results = []
    previous_text = ""
    current_text = ""
    
    for i, chunk in enumerate(chunks):
        previous_text = current_text
        current_text = previous_text + chunk
        
        result = parser.extract_tool_calls_streaming(
            previous_text=previous_text,
            current_text=current_text,
            delta_text=chunk,
            previous_token_ids=[],
            current_token_ids=[],
            delta_token_ids=[],
            request=request
        )
        results.append(result)
    
    # first 4 chunks should return None (no complete functions yet)
    assert results[0] is None
    assert results[1] is None
    assert results[2] is None
    assert results[3] is None
    
    # final chunk should return DeltaMessage with 2 tool calls
    assert results[4] is not None
    assert hasattr(results[4], 'tool_calls')
    assert len(results[4].tool_calls) == 2
    
    # verify the tool calls content
    tool_calls = results[4].tool_calls
    assert tool_calls[0].function.name == "search"
    assert '"query": "test"' in tool_calls[0].function.arguments
    assert '"limit": 5.0' in tool_calls[0].function.arguments
    
    assert tool_calls[1].function.name == "write_file"
    assert '"filename": "test.txt"' in tool_calls[1].function.arguments
    assert '"content": "hello"' in tool_calls[1].function.arguments

if __name__ == "__main__":
    test_incremental_parser()
    test_streaming_parser()