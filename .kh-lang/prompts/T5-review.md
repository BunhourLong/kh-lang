# T5 — review (reviewer, read-only)

Repo: /Users/capon/Documents/Projects/kh-lang

Review ONLY the uncommitted worktree changes for T5. Start with:
`git status --short` and `git diff` (tracked changes), then read the untracked files.
Read the brief `.kh-lang/prompts/T5.md`, `.kh-lang/tasks.md#T5`,
`.kh-lang/plan.md` D1/D5/D6/D7, `.kh-lang/spec.md` acceptance 1-6,
`.kh-lang/decisions.md`, and the public API of
`crates/kh-lang-core/src/{lib.rs,segmenter.rs,dictionary.rs}`.
Do NOT read `target/`, `.kh-lang/jobs/`, or `node_modules/`.

Do not edit any file. You may run read-only/verify commands (git, node, cargo) but
must not fix anything.

The builder reported one deliberate deviation, which you must assess:
- It accepts `Unknown<'_>` for options/words and parses/validates inside the function
  body, using `#[napi(ts_arg_type = "...")]`, because napi-rs's generated string-enum
  conversion throws `Status::InvalidArg` before the body and would bypass
  `ERR_KH_OPTIONS`. Check that the generated `index.d.ts` still exposes the intended
  types (`SegmentOptions`, `SegmenterOptions`, `SegmentMode`) and that runtime
  validation is complete.

Review for:
1. Scope: only the allowed files changed; core crate, data, scripts untouched; no
   T6 work (tests/README/CI) leaked in; generated `index.js`/`index.d.ts` consistent
   with `src/index.rs` and package.json.
2. JS surface exactly per spec/tasks: top-level `segment(text, opts?) -> string[]`,
   `normalize(text)`, class `Segmenter` with constructor, `segment`, `split`,
   `insertZwsp`, chainable `addWords`; `Token { text, start, end }` with UTF-16
   offsets; default mode bimm; `'bimm' | 'viterbi'` in d.ts.
3. Semantics: constructor `dictionary` and `addWords` are ADDITIVE to the embedded
   dictionary (not a replacement); `addWords` mutates the receiver and returns a
   chainable Segmenter; repeated `addWords` preserves earlier words and mode; viterbi
   and custom frequencies reach the core; duplicate word last-wins.
4. Validation / errors: `ERR_KH_OPTIONS` for bad/unknown mode or malformed options;
   `ERR_KH_WORDS` for empty/whitespace-only words, `\t`/`\r`/`\n`, non-finite,
   `<1` or `>u32::MAX` frequencies; fractional truncation; no `unwrap`/`expect`/
   panic paths; lone surrogates do not abort the process; JS errors expose `.code`.
5. Quality: glue contains no segmentation logic; napi config is correct (binaryName,
   targets, main/types/files, `--no-const-enum`); workspace root builds both crates;
   JSDoc present on all exports; no absolute paths or machine-specific hacks in
   generated `index.js`; package-lock is consistent.
6. Any BLOCKING correctness bug or spec violation, plus NITs.

Output: numbered BLOCKING findings first (with `file:line` and a one-line fix), then
NITs, then exactly one of `APPROVE` or `REQUEST-CHANGES`. Be specific; no general
praise. If you find zero BLOCKING issues, say so explicitly.
