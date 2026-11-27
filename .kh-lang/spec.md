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
8. Repo pushed to BunhourLong/kh-lang with split conventional commits and Delikit workflow.

## v0.1.1 hardening & correctness targets
1. Supply-chain hardened GitHub Actions: SHA-pinned third-party and first-party actions, `permissions: {}` default with least-privilege job scopes, `persist-credentials: false` on checkouts, `--ignore-scripts` on installs/publishes, release verification and binary-presence checks, Dependabot updates, and pinned dictionary source commit.
2. Complete Khmer combining-mark cluster handling (`U+17C9..=U+17D1`, `U+17D3`, `U+17DD` bound to the preceding base so OOV segmentation and `insert_zwsp` never split a diacritic from its consonant).
3. Strict plain-object validation for JS `dictionary` / `addWords` inputs (reject non-plain objects such as `Date`, `Map`, `RegExp`, or `Uint8Array` with `ERR_KH_OPTIONS` / `ERR_KH_WORDS`).
