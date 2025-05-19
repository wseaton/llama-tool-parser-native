1. developing

- install rust
- install `uv`

run: `uv run maturin develop`


2. using

`pip install llama-tool-parser-native`

Then:

```
vllm serve meta-llama/Llama-3.2-3B-Instruct \
            --port 8181 \
            --enable-auto-tool-choice \
            --chat-template tool_chat_template_llama3.2_pythonic.jinja \
            --tool-parser-plugin llama_tool_parser_native.NativePythonicToolParser \
            --tool-call-parser pythonic_native \
            --gpu-memory-utilization 0.99 \
            --enforce-eager \
            --max-model-len 32000 
```