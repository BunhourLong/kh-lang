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

## T7 review+publish — orchestrator
Reviewer subagent on whole diff; fix loop; split conventional commits; push to
BunhourLong/kh-lang (origin). No co-author.
-- STATE.md tracks status per task.
