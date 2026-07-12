#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
model_dir="$root/src/uar/runtime/matching/models"
base_url="https://huggingface.co/Xenova/bge-small-en-v1.5/resolve/main"

mkdir -p "$model_dir"
curl --fail --location --output "$model_dir/bg-small-en-v1.5.onnx" \
  "$base_url/onnx/model_quantized.onnx"
for file in config.json special_tokens_map.json tokenizer.json tokenizer_config.json; do
  curl --fail --location --output "$model_dir/$file" "$base_url/$file"
done

shasum -a 256 "$model_dir"/*
printf '%s\n' \
  "Review model and digest changes, run the local-model tests, and update catalog/SNAPSHOT.md."
