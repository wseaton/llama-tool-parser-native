1. developing

- install rust
- install `uv`

run: `cd python-bindings && uv run maturin develop`

tests: `uv run pytest -s -v`


2. using

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

3. TODOS

- [_] add support for streaming parsing (this should be possible with the `nom` backend)