# Llama Tool Parser (Native)

A native tool calling parser for Llama models, written in Rust and Python. Designed to be used with the `vllm` library for serving Llama models with tool calling capabilities.

Why? 

Regular expressions (and other naiive methods) are not great choices for parsing ASTs (or things that look like ASTs, such is the case for tool calling outputs in pythonic mode). The fact that LLMs can hallucinate and generate partial or otherwise malformed tool calls also has a [potential to cause security problems](https://github.com/vllm-project/vllm/security/advisories/GHSA-w6q7-j642-7c25) (DOS mostly). It is important that tool call parsers are fast, robust to failures and are reliable.

## Developing

- install rust
- install `uv`

run: `cd python-bindings && uv run maturin develop`

tests: `uv run pytest -s -v`


## Using

`pip install llama-tool-parser-native`

Then:

```
vllm serve meta-llama/Llama-3.2-3B-Instruct \
            --port 8181 \
            --enable-auto-tool-choice \
            --chat-template tool_chat_template_llama3.2_pythonic.jinja \
            --tool-parser-plugin python-bindings/pythonic_parser.py \ # or wherever you put this file
            --tool-call-parser pythonic_native \
            --gpu-memory-utilization 0.99 \
            --enforce-eager \
            --max-model-len 32000 
```

## Benchmarks

The tool calling parser has been tested for quality against the BFCLv3 leaderboard benchmark (`python` test category) and achieves the following results:

|Rank|Model                              |Live Overall Acc|AST Summary|Python Simple AST|Python Multiple AST|Python Parallel AST|Python Parallel Multiple AST|Irrelevance Detection|Relevance Detection|
|----|-----------------------------------|----------------|-----------|-----------------|-------------------|-------------------|----------------------------|---------------------|-------------------|
|1   |Llama-4-Scout-17B-16E-Instruct (FC)|58.69%          |75.57%     |81.40%           |74.36%             |75.00%             |66.67%                      |32.09%               |94.44%             |
|2   |Llama-3.2-3B-Instruct (FC)         |55.18%          |63.51%     |65.12%           |64.20%             |18.75%             |45.83%                      |41.72%               |88.89%             |

## Streaming Support

Yes, it supports streamed responses through SSE:

```
curl -X POST http://localhost:8181/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Accept: text/event-stream" \
  -d '{
    "model": "meta-llama/Llama-4-Scout-17B-16E-Instruct",
    "messages": [
      {
        "role": "user",
        "content": "What is the weather like in New York City?"
      }
    ],
    "tools": [
      {
        "type": "function",
        "function": {
          "name": "get_weather",
          "description": "Get the current weather for a given location",
          "parameters": {
            "type": "object",
            "properties": {
              "location": {
                "type": "string",
                "description": "The city and state, e.g. San Francisco, CA"
              },
              "unit": {
                "type": "string",
                "enum": ["celsius", "fahrenheit"],
                "description": "Temperature unit"
              }
            },
            "required": ["location"]
          }
        }
      }
    ],
    "stream": true,
    "temperature": 0.7,
    "max_tokens": 1000
  }'


  
data: {"id":"chatcmpl-0fa09c781bba4d66b457b734484ee08a","object":"chat.completion.chunk","created":1748465656,"model":"meta-llama/Llama-4-Scout-17B-16E-Instruct","choices":[{"index":0,"delta":{"role":"assistant","content":""},"logprobs":null,"finish_reason":null}]}

data: {"id":"chatcmpl-0fa09c781bba4d66b457b734484ee08a","object":"chat.completion.chunk","created":1748465656,"model":"meta-llama/Llama-4-Scout-17B-16E-Instruct","choices":[{"index":0,"delta":{"tool_calls":[{"id":"call_-1","type":"function","index":-1,"function":{"name":"get_weather","arguments":"{\"location\": \"New York City\"}"}}]}}]}

data: [DONE]
```