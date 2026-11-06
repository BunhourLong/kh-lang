# kh-lang — plan

## Architecture decisions (append to decisions.md; subagents reference, never edit)
- D1: Workspace: root Cargo.toml = napi glue crate (crate type cdylib, napi-rs v3);
  pure core at `crates/kh-lang-core`. Glue only translates — no segmentation logic there.
- D2: Core embeds dict via `include_str!("../../data/khmer.lbdict.txt")`; parse once
  lazily into `HashMap<String, u32>` (word → freq); entries with whitespace/ASCII were
  filtered at build time. Malformed lines skipped.
- D3: Pipeline: `normalize` (NFC, drop ZWSP U+200B, drop BOM, unify common Khmer
  variant chars if trivially safe) → split into runs: whitespace/punct run | Khmer run |
  Latin/digit run | other char. Segmentation applies ONLY to Khmer runs; other runs stay
  single tokens. Whitespace preserved as-is (losslessness criterion #2).
- D4: Modes: `bimm` — bidirectional maximal matching: forward longest match +
  backward longest match, take the path with fewer tokens; tie → forward (doc'd).
  `viterbi` — per-Khmer-run DP over dictionary with P(word) ∝ freq, OOV words get a
  small epsilon cost, whitespace-free runs. Fallback inside Khmer runs: if a whole run
  has no dictionary hit, emit single Khmer characters one by one (grapheme-safe: do not
  split after a base char that is followed by U+17D2 coeng or before combining marks).
- D5: Tokens carry UTF-16 code-unit start/end offsets (JS-native) computed in Rust.
- D6: Public surface (napi): `segment(text, options?) -> string[]`,
  `class Segmenter { segment(text): Token[]; split(text): Token[]; insertZwsp(text):
  string; addWords(words: string[] | Record<string, number>): void }`,
  `normalize(text): string`. `Token = { text: string; start: number; end: number }`.
  `Options = { mode?: 'bimm' | 'viterbi' }` for `segment`; Segmenter constructor takes
  `{ mode?, dictionary?: string[] | Record<string, number> }`. Sync functions (runs are
  fast; no async needed in v1).
- D7: Package: npm name `kh-lang`, root JS shim (index.js) with generated index.d.ts
  (napi generate), CI prebuilt matrix later; local dev via `npx napi build --platform`.

## Files to modify per task (overlap check)
- T2: crates/kh-lang-core/{Cargo.toml,src/lib.rs,src/normalizer.rs? — keys: mods in lib.rs,
  tests inline} (new files only)
- T3: crates/kh-lang-core/src/dictionary.rs, matcher.rs, lib.rs mod lines
- T4: crates/kh-lang-core/src/segmenter.rs, lib.rs mod lines
- T5: Cargo.toml (root), src/index.rs, package.json, index.js, tsconfig.json
- T6: test/index.test.mjs, .github/workflows/ci.yml, README.md
No two tasks share files beyond lib.rs mod lines → sequential dispatch T2→T3→T4→T5→T6.
