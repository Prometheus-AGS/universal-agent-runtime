# Provider catalog snapshot

- File: `provider_catalog.json`
- Snapshot date: 2026-08-28
- SHA-256: `898786703b804218bd4acc54a624a85832f16bc2ae16ab4cddd5fa7c59babca3`
- Entries: 322 providers
- Sources:
  - Know-Me-Tools `models.dev` catalog, commit `f97df19af40bc322ccbffc91138f360154940a63`
  - liter-llm provider and catalog schemas, commit `c5c6caac617eb931cd5009146a70831422ec236c`

The file is the reviewed merged output previously produced by the UAR catalog
builder. Release builds copy it verbatim and perform no network access. Refresh
is an explicit maintainer operation from the pinned submodules, documented in
`docs/build-reproducibility.md`.

## Packaged local model inputs

Source: `Xenova/bge-small-en-v1.5`, ONNX quantized model and tokenizer assets.

| File | SHA-256 |
|---|---|
| `bg-small-en-v1.5.onnx` | `6c9c6101a956d62dfb5e7190c538226c0c5bb9cb27b651234b6df063ee7dbfe4` |
| `config.json` | `f2c87ea17fdfa286f48d6e1506f2cdc838d303420dc55606190e67e5acd9e186` |
| `special_tokens_map.json` | `b6d346be366a7d1d48332dbc9fdf3bf8960b5d879522b7799ddba59e76237ee3` |
| `tokenizer.json` | `d241a60d5e8f04cc1b2b3e9ef7a4921b27bf526d9f6050ab90f9267a1f9e5c66` |
| `tokenizer_config.json` | `9261e7d79b44c8195c1cada2b453e55b00aeb81e907a6664974b4d7776172ab3` |
