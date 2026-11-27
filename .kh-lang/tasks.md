# tasks

## T1 data — DONE (orchestrator)
Dictionary prepared; `data/khmer.lbdict.txt` (33,895 entries) + build script + license.

## T2 core-foundation — implementer
Create `crates/kh-lang-core` (new crate, edition 2024, no deps; e.g. width-fun / unicode-width-not needed).
Modules & items (doc-commented, unit tests inline, `#![deny(missing_docs)]`):
- `normalizer::normalize(&str) -> Cow<str>`: NFC-equivalent cleanup WITHOUT external crates:
  rebuild string dropping U+200B ZWSP, U+FEFF BOM, and reordering combining Khmer marks
  only if they appear out of canonical order (leave exotic cases untouched — do NOT
  attempt full NFD/NFC; document why). Risk: NFC over-engineering — forbid: politely refuse via push-back rule.
- `cluster::ClusterIter<'a>`: Khmer grapheme-safe cluster iterator: iterate chars but
  bind `(U+17D2 coeng + base)` pairs and attach following combining marks (U+17B6..U+17C5
  vowel signs, U+17C6 nikhahit, shifter U+17B7..?) — safest rule: a cluster = strongest
  char (consonant/vowel) + all following [\u{17B6}-\u{17C5}\u{17C6}\u{17C7}\u{17C8}] and a
  coeng+base pair kept together; non-Khmer chars are single-char clusters.
- `run::runs(text) -> Vec<Range<usize>>` classify chars into Khmer / non-Khmer runs
  (merge contiguous same-class).
Tests: losslessness of normalizer w.r.t. visible chars (zwsp removed), cluster binding cases
("ខ្ញ" coeng, "ខ្ញុំ", English "abc", mixed runs).
Verification: `cargo fmt --all --check && cargo clippy --all-targets -- -D warnings && cargo test`

## T3 core-matchers — implementer (after T2)
- `dictionary::Dictionary::embedded() -> &'static Dictionary` (OnceLock),
  `Dictionary::from_text(&str)`, `.insert(word, freq)`, `.contains`, `.freq`,
  `add`/`add_words(&[&str])`. Parse: `word\tfreq`, unknown freq → skip default freq 10_000; malformed → skip.
- `matcher::bimm(&Dictionary, &str) -> Vec<Range<usize>>`: forward maximal match +
  backward maximal match, dynamic programming pick of fewer tokens; ties → prefer forward path.
  Longest match against dictionary only; OOV → single-cluster fallback.
- `matcher::viterbi(&Dictionary, &str) -> Vec<Range<usize>>`: HMM-lite DP: score(word) =
  log(freq / Σfreq) as f64; unseen-word emissions use a small constant cost so OOV is
  handled by splitting, never by discarding chars.
Tests: known examples ខ្ញុំ/ស្រលាញ់/ភ្នំពេញ; bimm and viterbi both yield the 3 dictionary
compounds for "សូមស្វាគមន៍មកកាន់ប្រទេសកម្ពុជា"; OOV garbage string returns one token per
cluster and preserves losslessness; mixed digits/ZWSP/emoji/trailing-coeng is lossless.
Verification: same as T2.

## T4 core-segmenter — DONE
`Segmenter` type in `crates/kh-lang-core/src/segmenter.rs`:
- `Segmenter::new()` (embedded dict, Bimm), `.builder()` -> `SegmenterBuilder`
  (mode, dictionary override, extra words).
- `pub enum Mode { Bimm, Viterbi }`, `pub struct Token { text, start: usize(bytes), end }`?
  finalize: public `Segmenter::segment(&self, &str) -> Vec<Token>` where Token { text:
  String, start: u32, end: u32 } — offsets in UTF-16 code units (compute at call time);
  plus `fn insert_zwsp(&self, &str) -> String`.
Runs: whitespace/punct/Latin/digit runs = single tokens (run::runs from T2).
Property test: round-trip `concat(segment(s))) == s` (use fixed pseudo-random fixture cases, no external crates).
Verification: same as T2.

## T5 napi-glue — implementer (after T4)
Root `Cargo.toml` + `src/index.rs` napi exports per plan.md D6, register, `Token` class
struct (#[napi(object)]), Options handling napi::Error with `.code` (e.g. `ERR_KH_DICT`),
package.json (name kh-lang, napi config targets native; scripts build/test/bench stub),
index.js shim once built, tsconfig, index.d.ts generated via `npx napi generate`.
Verify: `cargo clippy && cargo build -p kh-lang && npx napi build --platform && node -e
"const k=require('./index.js');console.log(k.segment('ខ្ញុំស្រលាញ់ភ្នំពេញ'))"` (PATH=$HOME/.cargo/bin:$PATH)

## T6 tests+ci+docs — implementer (after T5)
`test/*.test.mjs` node:test suite covering spec.md acceptance list, README.md
(bilingual short EN + KH snippet), LICENSE MIT, .github/workflows/ci.yml (fmt→clippy→test
→napi build matrix→jest-free npm test), .npmignore, npm pack dry-run check.
Verify: `npm test` green from clean clone.

## T7 review+publish — DONE (v0.1.0 live on npm)
Reviewer subagent on whole diff; fix loop; split conventional commits; push to
BunhourLong/kh-lang (origin). No co-author.
T7a: Delikit refresh (`.claude/run-task.md`, `AGENTS.md`, remove legacy runners), GitHub Actions & supply-chain hardening, and complete `v0.1.0` npm publish (all 6 packages live on npm at `0.1.0`).

---

## Milestone v0.1.1 — Khmer correctness & JS input hardening

### T8 v0.1.1-core-correctness — next
Allowed files: `crates/kh-lang-core/src/{cluster,run,segmenter}.rs`, `test/segment.test.mjs`, `test/gold-100.test.mjs` (read-only verification unless invariant changes).
1. Extend `is_khmer_mark` in `crates/kh-lang-core/src/cluster.rs` to cover `U+17B6..=U+17D1`, `U+17D3`, and `U+17DD` (and evaluate `U+17B4..=U+17B5`), keeping OOV diacritics (`គ់`, `គ៉ា`, `គ៍`, `ក័ង`) bound to their base consonants.
2. Group contiguous Khmer digits (`U+17E0..=U+17E9`) into a single token (e.g., `ឆ្នាំ២០២៦` → `["ឆ្នាំ", "២០២៦"]`, `តម្លៃ១០០០៛` → `["តម្លៃ", "១០០០", "៛"]`), preserving exact losslessness and UTF-16 offsets.
3. Suppress `insert_zwsp` before Khmer closing punctuation/signs (`។` `U+17D4`, `៕` `U+17D5`, `៖` `U+17D6`, `ៗ` `U+17D7`, `៚` `U+17DA`, `៛` `U+17DB`) so browsers/editors never orphan punctuation onto a new line.
4. Add regression tests in Rust and `test/segment.test.mjs`; verify `test/gold-100.test.mjs` boundary F1 stays green (`viterbi >= bimm`).

### T9 v0.1.1-js-validation-and-release
Allowed files: `src/index.rs`, `test/api.test.mjs`, `README.md`, and version bump files (`Cargo.toml`, `crates/kh-lang-core/Cargo.toml`, `package.json`, `npm/*/package.json`, `index.js`) when releasing `v0.1.1`.
1. Reject non-plain JS objects (`new Date()`, `new Map()`, `/regex/`, `Buffer.from('x')`, `new Uint8Array()`) passed as `options`, `dictionary`, or `addWords` with `ERR_KH_OPTIONS` / `ERR_KH_WORDS` (keep plain `{}` literals and `Object.create(null)` accepted).
2. Clarify or fix `Segmenter#addWords` chaining identity semantics (`this` return vs documented copy behavior) with regression tests in `test/api.test.mjs` and README docs.
3. Run fresh-context high-risk review + branch `security-audit`, commit, tag, and publish `v0.1.1`.

---

## Milestone v0.2.0 — Performance & binary size

### T10 v0.2.0-dictionary-overlay-and-prefix-lookup
Allowed files: `crates/kh-lang-core/src/{dictionary,matcher,segmenter}.rs`, `src/index.rs`, `test/*.test.mjs`.
1. Replace the 33,895-entry `Dictionary` deep-clone on custom words (`new Segmenter({ dictionary })` / `.addWords()`) with a lightweight overlay over `&'static Dictionary`.
2. Add zero-dependency prefix-pruned dictionary lookup so `bimm` and `viterbi` stop scanning as soon as no dictionary word shares the prefix; preserve determinism, losslessness, and `test/gold-100.test.mjs` F1.

### T11 v0.2.0-release-profile-and-benchmarks
Allowed files: `Cargo.toml`, `bench/*` (new), `README.md`, and version bump files for `v0.2.0`.
1. Add `[profile.release]` (`lto = true`, `codegen-units = 1`, `strip = "symbols"`) in root `Cargo.toml` and verify `.node` binary size reduction.
2. Add reproducible zero-dependency JS + Rust throughput/instantiation benchmarks (`bench/bench.mjs`) and document benchmark numbers in `README.md`.
3. Run fresh-context high-risk review + branch `security-audit`, commit, tag, and publish `v0.2.0`.

---

## Milestone v0.3.0 — Platform reach, ecosystem & cohesive APIs

### T12 v0.3.0-musl-win-arm64-and-wasm
Allowed files: `package.json`, `npm/*`, `.github/workflows/{ci,release}.yml`, `index.js`, `index.d.ts`, `README.md`.
1. Add Linux musl (`x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`), Windows ARM64 (`aarch64-pc-windows-msvc`), and WASM/edge fallback (`wasm32-wasi`) targets with CI/release builds and smoke tests.

### T13 v0.3.0-crates-io-and-js-api
Allowed files: `crates/kh-lang-core/{Cargo.toml,README.md,src/*}`, `src/index.rs`, `package.json`, `index.d.ts`, `test/*.test.mjs`, `README.md`, and version bump files for `v0.3.0`.
1. Prepare `kh-lang-core` metadata and crate README for publishing on crates.io (`repository`, `homepage`, `readme`, `keywords`, `categories`).
2. Add an explicit `"exports"` map to `package.json` for strict ESM/CJS/`NodeNext` TypeScript resolution.
3. Expose cohesive Rust + JS APIs: `clusters(text)` (grapheme-safe Khmer clusters), `khmerDigitsToAscii` / `asciiDigitsToKhmer`, and `splitSentences(text)` (`។`/`៕` + standard punctuation), with full Rust + Node + TypeScript tests.
4. Run fresh-context high-risk review + branch `security-audit`, commit, tag, and publish `v0.3.0`.

---

## Milestone v0.4.0 — Real-world corpus, OSS governance & OIDC release

### T14 v0.4.0-real-world-corpus-and-dict
Allowed files: `test/fixtures/real-khmer.mjs` (new), `test/real-khmer.test.mjs` (new), `scripts/build-dict.mjs`, `data/*`, `README.md`.
1. Add a curated real-world Khmer evaluation fixture (news, colloquial text, modern tech/loanwords, dates/prices, abbreviations) and a clean dictionary supplement layer separated from upstream SIL `khmerlbdict`.

### T15 v0.4.0-oss-governance-and-oidc
Allowed files: `CHANGELOG.md`, `SECURITY.md`, `CONTRIBUTING.md`, `README.md`, `.github/workflows/{ci,release}.yml`.
1. Add `CHANGELOG.md`, `SECURITY.md`, `CONTRIBUTING.md`, README status badges (npm, CI, license, SLSA provenance), and an isolated `github-release` job (`permissions: { contents: write }` on that job only) in `release.yml`.
2. Upgrade SHA-pinned GitHub Actions off deprecated Node 20 runtime majors.
3. Migrate `release.yml` to npm Trusted Publishing (OIDC) once enabled on npmjs.com (removing static `NPM_TOKEN`), and enable/document GitHub repo protections (`main` branch ruleset, `v*` tag ruleset, Dependabot security alerts).

---

Verification (all tasks):
`export PATH="$HOME/.cargo/bin:$PATH" && cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && npm test && npx tsc --noEmit`
-- STATE.md tracks status per task.
