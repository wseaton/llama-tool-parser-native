import json
from collections.abc import Sequence
from typing import Union

from transformers import PreTrainedTokenizerBase

from vllm.entrypoints.openai.protocol import (
    ChatCompletionRequest,
    DeltaMessage,
    ExtractedToolCallInformation,
    FunctionCall,
    ToolCall,
)
from vllm.entrypoints.openai.tool_parsers.abstract_tool_parser import (
    ToolParser,
    ToolParserManager,
)
from vllm.logger import init_logger
from typing import List

from .llama_tool_parser_native import parse_tools

logger = init_logger(__name__)


class _UnexpectedAstError(Exception):
    pass


@ToolParserManager.register_module("pythonic_native")
class NativePythonicToolParser(ToolParser):
    """
    Tool call parser for models that produce tool calls in a pythonic style,
    such as Llama 3.2 and Llama 4 models.

    Used when --enable-auto-tool-choice --tool-call-parser pythonic_native are all set
    """

    def __init__(self, tokenizer: PreTrainedTokenizerBase):
        super().__init__(tokenizer)

    # Rename for readability. This is NOT a tool id.
    @property
    def current_tool_index(self) -> int:
        return self.current_tool_id

    @current_tool_index.setter
    def current_tool_index(self, value: int) -> None:
        self.current_tool_id = value

    @staticmethod
    def _process_tool_arguments(kwargs: dict) -> dict:
        """
        Process tool arguments to handle single-value dictionaries.
        For dictionaries with a single key-value pair, extract just the value.

        This function recursively processes dictionaries to extract values from
        single-key dictionaries at any nesting level.
        """
        result = {}
        for k, v in kwargs.items():
            if isinstance(v, dict):
                if len(v) == 1:
                    # For a single key-value dict, extract the value
                    result[k] = next(iter(v.values()))
                else:
                    # For multi-key dicts, process recursively
                    result[k] = NativePythonicToolParser._process_tool_arguments(v)
            else:
                # Non-dict values remain unchanged
                result[k] = v
        return result

    def extract_tool_calls(
        self, model_output: str, request: ChatCompletionRequest
    ) -> ExtractedToolCallInformation:
        """
        Extract the tool calls from a complete model response.
        """

        print(f"!!! model_output {model_output}")

        extracted_tool_calls: List[dict] = parse_tools(model_output)

        if not extracted_tool_calls:
            # No tool calls found, return the entire model output as content
            # and set tools_called to False.
            logger.warning(f"!!! no tool calls found in output:\n{model_output}\n")
            # This is a fallback for when the regex fails to match.
            # We still want to return the model output as content.
            # This is a workaround for the case where the model output is
            # not a valid Python list of function calls.
            return ExtractedToolCallInformation(
                tools_called=False, tool_calls=[], content=model_output
            )

        return ExtractedToolCallInformation(
            tools_called=True,
            tool_calls=[
                ToolCall(
                    type="function",
                    function=FunctionCall(
                        name=tool["name"],
                        arguments=json.dumps(
                            self._process_tool_arguments(tool["kwargs"])
                        ),
                    ),
                )
                for tool in extracted_tool_calls
            ],
            content=model_output,
        )

    def extract_tool_calls_streaming(
        self,
        previous_text: str,
        current_text: str,
        delta_text: str,
        previous_token_ids: Sequence[int],
        current_token_ids: Sequence[int],
        delta_token_ids: Sequence[int],
        request: ChatCompletionRequest,
    ) -> Union[DeltaMessage, None]:
        raise NotImplementedError(
            "Streaming tool call extraction is not yet implemented for NativePythonicToolParser."
        )
