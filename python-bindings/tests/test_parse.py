from llama_tool_parser_native import parse_tools
import pytest


@pytest.mark.parametrize("engine", ["nom", "logos"])
def test_parse_python(engine):
    # Test with a simple function
    code = """sometext<|python_start|>[
     reschedule_event(event_identifier="your_event_id", new_datetime="next_friday_date_in_iso_format"),
     reschedule_event(event_identifier="12345", new_datetime="2024-09-27T00:00:00Z"),
     reschedule_event(event_identifier="12345", new_datetime="2024-09-27T00:00:00Z"),
     reschedule_event(event_identifier="12345", new_datetime="2024-09-27T00:00:00Z"),
     reschedule_event(event_identifier="12345", new_datetime="2024-09-27T00:00:00Z"),
     reschedule_event(event_identifier="12345", new_datetime="2024-09-27T00:00:00Z"),
     reschedule_event(event_identifier="12345", new_datetime="2024-09-27T00:00:00Z")
   ]<|python_end|>
     sometext"""

    # time the parsing
    import time

    start_time = time.time()
    tools = parse_tools(code, engine=engine)
    end_time = time.time()
    print(f"Parsing took {(end_time - start_time) * 1000:.2f} milliseconds")
    print(tools)
    # Check if the output is a list
    assert isinstance(tools, list)
