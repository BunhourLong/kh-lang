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
- D8: Workflow & release hardening: Delikit (`.claude/run-task.md` inheriting `~/.agents/routing.md`), SHA-pinned GitHub Actions with least-privilege permissions and `--ignore-scripts`, Dependabot, pinned SIL dictionary source commit.

## Files to modify per task (overlap check)
- T2: crates/kh-lang-core/{Cargo.toml,src/lib.rs,src/normalizer.rs? — keys: mods in lib.rs,
  tests inline} (new files only)
- T3: crates/kh-lang-core/src/dictionary.rs, matcher.rs, lib.rs mod lines
- T4: crates/kh-lang-core/src/segmenter.rs, lib.rs mod lines
- T5: Cargo.toml (root), src/index.rs, package.json, index.js, tsconfig.json
- T6: test/index.test.mjs, .github/workflows/ci.yml, README.md
- T8 (v0.1.1): crates/kh-lang-core/src/{cluster,run,segmenter}.rs, test/segment.test.mjs
- T9 (v0.1.1): src/index.rs, test/api.test.mjs, README.md, version bump files
- T10 (v0.2.0): crates/kh-lang-core/src/{dictionary,matcher,segmenter}.rs, src/index.rs, test/*.test.mjs
- T11 (v0.2.0): Cargo.toml, bench/*, README.md, version bump files
- T12 (v0.3.0): package.json, npm/*, .github/workflows/{ci,release}.yml, index.js, index.d.ts, README.md
- T13 (v0.3.0): crates/kh-lang-core/{Cargo.toml,README.md,src/*}, src/index.rs, package.json, index.d.ts, test/*.test.mjs, README.md, version bump files
- T14 (v0.4.0): test/fixtures/real-khmer.mjs, test/real-khmer.test.mjs, scripts/build-dict.mjs, data/*, README.md
- T15 (v0.4.0): CHANGELOG.md, SECURITY.md, CONTRIBUTING.md, README.md, .github/workflows/{ci,release}.yml
T2–T6 ran sequentially; T8–T15 execute sequentially one task per session across milestones (v0.1.1 → v0.2.0 → v0.3.0 → v0.4.0).
