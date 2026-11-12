# Shared buffer memory mapping dependency

Base crates.io shared-buffer 0.1.4, SHA-256
f6c99835bad52957e7aa241d3975ed17c1e5f8c92026377d117a606f36b84b16.
Select memmap2 0.9.12, retaining the read-only mapping API and existing
ownership and slicing. This independent Wasmer graph previously selected
memmap2 0.6.2.

- https://docs.rs/crate/memmap2/0.9.12
- https://rustsec.org/advisories/RUSTSEC-2026-0186.html
