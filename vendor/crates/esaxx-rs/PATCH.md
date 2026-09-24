# Windows dynamic CRT patch

This is the crates.io `esaxx-rs` 0.1.10 source with the MSVC runtime changed
from static (`/MT`) to dynamic (`/MD`) in `build.rs`.

The Boss sidecar Windows build reached its final link in GitHub Actions run
35949935340, where `esaxx.o` reported `MT_StaticRelease` while
`kreuzberg-tesseract` reported `MD_DynamicRelease`, producing LNK2038 and
LNK1319. This patch applies the same source change proposed upstream in
Narsil/esaxx-rs PR #19 (commit
`4f4fd5acd624e43c770a0d198fcaa9d97bfd7f0b`) and keeps the release source
closure local and immutable.
