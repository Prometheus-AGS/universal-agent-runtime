# Provider catalog snapshot

- File: `provider_catalog.json`
- Snapshot date: 2026-07-24
- SHA-256: `4b613e87de884e5741aaa281fc64e16c8fe280bc6c769fb6d2f89d1b3ba5afe1`
- Entries: 297 providers
- Sources:
  - `https://models.dev/api.json`
  - liter-llm provider schemas, commit `c37e98411ff154fb2c256856428c15d7340b1325`

The file is the reviewed merged output previously produced by the UAR catalog
builder. Release builds copy it verbatim and perform no network access. Refresh
is an explicit maintainer operation documented in `docs/build-reproducibility.md`.

## Packaged local model inputs

Source: `Xenova/bge-small-en-v1.5`, ONNX quantized model and tokenizer assets.

| File | SHA-256 |
|---|---|
| `bg-small-en-v1.5.onnx` | `6c9c6101a956d62dfb5e7190c538226c0c5bb9cb27b651234b6df063ee7dbfe4` |
| `config.json` | `f2c87ea17fdfa286f48d6e1506f2cdc838d303420dc55606190e67e5acd9e186` |
| `special_tokens_map.json` | `b6d346be366a7d1d48332dbc9fdf3bf8960b5d879522b7799ddba59e76237ee3` |
| `tokenizer.json` | `d241a60d5e8f04cc1b2b3e9ef7a4921b27bf526d9f6050ab90f9267a1f9e5c66` |
| `tokenizer_config.json` | `9261e7d79b44c8195c1cada2b453e55b00aeb81e907a6664974b4d7776172ab3` |
