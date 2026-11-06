# kh-lang — spec

## Intent
Official ready-to-use npm package `kh-lang` for Khmer word segmentation, implemented in
Rust (napi-rs prebuilt binaries). Serves anyone processing Khmer text worldwide.

## Non-goals
No model training/ML runtimes, no HTTP services, no browser WASM in v1 (napi native only;
wasi target added later via napi.targets), no Thai/Lao support.

## Acceptance criteria (pass/fail)
1. `segment("ខ្ញុំស្រលាញ់ភ្នំពេញ")` → `["ខ្ញុំ", "ស្រលាញ់", "ភ្នំពេញ"]` (or valid segmentation
   of the same string with no lost characters — concatenation of tokens equals the input).
2. Segmentation is lossless: concatenating returned tokens always reproduces the input
   exactly, for any input (Khmer, Latin, digits, whitespace, emoji, ZWSP).
3. Default mode deterministic `bimm` (bidirectional maximal matching); optional
   `viterbi` mode using dictionary frequencies shows ≥ equal F1 on a 100-sentence
   hand-checked sample vs bimm in repo tests.
4. Library never panics on arbitrary input, including invalid-truncated UTF-16 -> Rust
   boundary cases; malformed dictionary lines are skipped, not fatal.
5. `Segmenter` supports custom words (string[] or {word: freq}), and method chaining:
   `.addWords()`.
6. Public API: `segment(text, opts?)`, `Segmenter` class (`.segment(text)`,
   `.split(text)` → tokens with UTF-16 offsets, `.insertZwsp(text)`, `.addWords`),
   `normalize(text)`. TypeScript types shipped; JSDoc on all exports.
7. `cargo fmt --check && cargo clippy -D warnings && cargo test` green; npm install
   works from a clean clone (`napi build && node test/index.spec.mjs`).
8. Repo pushed to BunhourLong/kh-lang with split conventional commits and this workflow.
