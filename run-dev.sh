#!/bin/bash
cd /home/gqadonis/Projects/prometheus/universal-agent-runtime
export LLM_BASE_URL='https://api.openai.com'
export LLM_MODEL='gpt-5.2'
export LLM_API_KEY="${LLM_API_KEY:-your-openai-api-key-here}"  # set LLM_API_KEY in your environment
exec ./target/debug/universal-agent-runtime --config config.embedded.yaml
