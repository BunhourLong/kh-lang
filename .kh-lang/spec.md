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

## Milestone roadmap (post-v0.1.0)
- **v0.1.1 — Khmer correctness & JS input hardening:**
  1. Supply-chain hardened GitHub Actions (SHA-pinned actions, `permissions: {}` default, `persist-credentials: false`, `--ignore-scripts`, release verify & `.node` presence checks, Dependabot, pinned dictionary source commit — already landed on `main`).
  2. Complete Khmer combining-mark cluster handling (`U+17C9..=U+17D1`, `U+17D3`, `U+17DD`), contiguous Khmer digit grouping (`U+17E0..=U+17E9`), and closing-punctuation protection in `insert_zwsp`.
  3. Strict plain-object validation for JS `options` / `dictionary` / `addWords` inputs and explicit `addWords` identity semantics.
- **v0.2.0 — Performance & binary size:**
  1. Custom-word overlay over `&'static Dictionary` (no 33,895-entry clone) and zero-dep prefix-pruned dictionary lookup.
  2. `[profile.release]` (`lto`, `codegen-units = 1`, `strip = "symbols"`) plus reproducible JS + Rust benchmarks.
- **v0.3.0 — Platform reach, ecosystem & cohesive APIs:**
  1. Add Linux `musl` (`x64`/`arm64`), Windows `arm64`, and `wasm32-wasi` targets.
  2. Prepare `kh-lang-core` for crates.io, add `package.json` `"exports"`, and expose `clusters`, Khmer/ASCII digit helpers, and `splitSentences`.
- **v0.4.0 — Real-world corpus, OSS governance & OIDC release:**
  1. Curated real-world Khmer test corpus and clean dictionary supplement layer.
  2. `CHANGELOG.md`, `SECURITY.md`, `CONTRIBUTING.md`, GitHub Releases automation, Node 24-ready SHA-pinned actions, npm Trusted Publishing (OIDC), and GitHub branch/tag rulesets.
