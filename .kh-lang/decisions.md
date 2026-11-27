# decisions (append-only; orchestrator writes, subagents read)

## 2026-11-06 — T3 compound segmentation
The embedded SIL dictionary contains `សូមស្វាគមន៍` (freq 46), `មកកាន់` (69), and
`ប្រទេសកម្ពុជា` (577). On `សូមស្វាគមន៍មកកាន់ប្រទេសកម្ពុជា` both bimm and viterbi
therefore produce the correct 3-token output. The earlier brief's ">= 4 tokens" and
"bimm == viterbi on arbitrary samples" were arbitrary; the test now pins the exact
3 compounds and both algorithms' agreement. Do not force a 4th token.

Reviewer follow-ups accepted: OOV test asserts one token per cluster; mixed fixture
(digits, ZWSP, emoji, trailing coeng) asserts losslessness.

## 2026-11-06 — T4 segmenter semantics
`segment` runs on the raw input, never on `normalize` output: dropping ZWSP/BOM would
violate acceptance #2 (`concat(tokens) == original input`). ZWSP/BOM stay inside
`Other` tokens; `normalize` remains a separate public API. `insert_zwsp` inserts U+200B
only between adjacent Khmer-Khmer token boundaries, skips a left token ending in coeng
U+17D2, and skips Khmer digit pairs U+17E0..U+17E9.

## 2026-11-06 — Workflow runner
`opencode serve` + `opencode run --attach` died with EPROTO/TLS on long jobs on this
machine. The runner now executes the job directly inside the cmux split and signals
completion via `.kh-lang/jobs/ai-<role>.done`; visibility and fresh-context isolation
are unchanged. Jobs past ~30k context are also avoided by keeping briefs tight.

## 2026-11-06 — T5 JS surface semantics
- `dictionary` in the `Segmenter` constructor and `addWords` are ADDITIVE on top of
  the embedded dictionary (spec calls them "custom words"); v1 has no JS API to
  replace the base table.
- `addWords` mutates the receiver and returns a Segmenter so
  `new Segmenter().addWords([...]).segment(...)` chains (spec #5). Returning a cheap
  copy is acceptable; object identity is not guaranteed. Returning `void` would
  violate spec #5.
- Class `.segment` and `.split` both return `Token[]` (UTF-16 offsets); top-level
  `segment` returns `string[]`. `.split` is documented as an alias.
- Stable error codes: `ERR_KH_OPTIONS` (invalid mode/options), `ERR_KH_WORDS` (empty
  or whitespace-only word, tab/CR/LF in word, non-finite/non-positive/too-large
  frequency). Invalid frequencies and words must throw, never panic.
- Generated `index.js` / `index.d.ts` are committed package artifacts;
  `napi build --platform --no-const-enum` emits string-literal unions instead of
  const enums.

## 2026-11-06 — T5 review outcome
- napi-rs converts string enums/typed option objects before the Rust function body,
  which would bypass custom codes. T5 therefore accepts `Unknown` for options/words
  and validates inside the body, keeping the generated d.ts typed via `ts_arg_type`.
  Reviewer accepted the deviation; `ERR_KH_OPTIONS` / `ERR_KH_WORDS` are stable.
- `addWords` shares the built core segmenter via `Arc` and rebuilds from the custom
  word list on each call, which deep-clones the 33,895-entry embedded table. Fine for
  occasional calls; add a core `(word, freq)` builder hook if it becomes hot.
- Known NIT: a non-array object with a custom prototype (e.g. `Map`, `Date`) is read
  as an empty word map instead of throwing. Tighten when hardening JS input parsing.
- Reviewer APPROVE (0 BLOCKING) after one fix round: removed a JS-length-driven
  `Vec::with_capacity`, corrected `normalize` docs, added own-property guards against
  prototype pollution. Tester ALL PASS.

## 2026-11-06 — T6 tests/CI/docs
- `node:test` acceptance suite (no jest/vitest): exact segmentation, losslessness in
  both modes, UTF-16 offsets, `insertZwsp`, custom words/chaining, stable error codes,
  malformed-UTF-16 no-abort, package type surface.
- Gold fixture: 100 unique pure-Khmer sentences built from embedded-dictionary
  headwords (gold by construction, every token dictionary-verified), fixed seed,
  25 bimm/viterbi disagreement sentences, selected without correctness filtering.
  F1: bimm 0.9439, viterbi 0.9847 (viterbi >= bimm).
- N-API converts lone/unpaired UTF-16 surrogates to U+FFFD, so losslessness is
  guaranteed for well-formed strings only; `segment("\uD800")` never panics
  (spec #4). Documented in README.
- CI: Rust fmt/clippy/test; native build matrix 3 OSes x Node 18/20; `npm test`
  self-contained via `pretest`; generator determinism guard; `npm pack` asserts
  JS/d.ts/README/LICENSE/licenses/*.node and no test/.kh-lang/target artifacts.
- Known NIT accepted: `.npmignore` is advisory because package.json `files` is
  authoritative; the pack assertion is the real guard.
- Workflow: `run-agent.sh` prints the brief, titles the session, streams tool
  activity, and prints `opencode --session <id>` so jobs are inspectable in OpenCode.
- Ponytail is now required before code work (global AGENTS.md) and applied in the
  builder/reviewer agent instructions, subordinate to the quality rules.

## 2026-11-27 — Delikit refresh, CI/release hardening, and v0.1.0/v0.1.1 split
- Migrated project workflow to `delikit` via `.claude/run-task.md` (inheriting `~/.agents/routing.md`) and updated `AGENTS.md`; removed legacy `scripts/run-agent.sh`, `opencode.json`, `.opencode/agents/*`, and `.kh-lang/prompts/*.md`.
- Hardened `.github/workflows/ci.yml` and `.github/workflows/release.yml` (40-char commit SHA pins for all actions including `dtolnay/rust-toolchain`, top-level `permissions: {}` with least-privilege job scopes, `persist-credentials: false`, `--ignore-scripts`, removed `cache: npm` from `release.yml`, added `verify` job and `.node` binary-presence guard before `npm publish`, sorted directory enumeration and exact `npm view` version matching), added `.github/dependabot.yml`, and pinned `scripts/fetch-dict.sh` to `silnrsi/khmerlbdict` commit `54abc6b87f4ffa6a0c95c09836c7c86648002273`.
- Version-skew guard: npm support cleared `kh-lang-win32-x64-msvc`, while the 4 macOS/Linux `0.1.0` packages are already live and immutable on npm. Rust/FFI fixes (`T8`: missing Khmer combining marks in `cluster.rs` and strict plain-object validation in `src/index.rs`) are therefore queued for `v0.1.1` after `v0.1.0` completes without binary skew.
- Governance & security gates: added root `CLAUDE.md` importing `@AGENTS.md`, added the codebase-as-memory banned-patterns policy (fix root causes, enforce deterministically, ban bad patterns in `AGENTS.md`), enforced strict local-commit-until-explicit-user-push discipline, and required a pre-push/rollout `security-audit` pen-test on `claudeAgent_2` / `claude-opus-5-5` (`effort: high`) before pushing to `main` or releasing.

## 2026-11-27 — T8 core correctness
- Extended `is_khmer_mark` in `cluster.rs` to `U+17B4..=U+17D1`, `U+17D3`, and `U+17DD` (including `U+17B4..=U+17B5`, which are Unicode `Mn` / `Extend` marks with zero embedded-dictionary hits) and fixed cluster-initial coeng (`U+17D2`) forward binding with `.get(..)` slicing.
- Added `RunKind::KhmerDigit` (`U+17E0..=U+17E9`) in `run.rs` so contiguous Khmer digits emit as one token before dictionary matching, preventing SIL Bible digit entries (e.g. `ទី១`) from splitting numbers like `ទី១០` while keeping Khmer digits separate from `Other` runs and preserving ZWSP between Khmer words and digits.
- Suppressed `insert_zwsp` before closing signs (`។` `U+17D4`, `៕` `U+17D5`, `៖` `U+17D6`, `ៗ` `U+17D7`, `៚` `U+17DA`, `៛` `U+17DB`), marks, and coeng while keeping ZWSP allowed at token boundaries after closing signs (never splitting inside an atomic dictionary token). `write_khmer_run_with_zwsp` iteratively re-segments multi-token suppressed chunks and Viterbi chunks to a fixed point so `insert_zwsp` is strictly idempotent across custom closing-sign-initial words and IEEE-754 rounding ties (prioritizing strict idempotency and full-run segmentation over strict subset-of-`segment` boundaries on synthetic edge cases).
- Deterministically regenerated `test/fixtures/gold-100.mjs` via `node scripts/generate-gold.mjs` after fixing `U+17C9` resolved BIMM's split on `យូររបៀបថៃក្រម៉ា` (restoring 25 mode disagreements without weakening `test/gold-100.test.mjs`, and pinning `យូររបៀបថៃក្រម៉ា` explicitly in Rust and JS tests).
