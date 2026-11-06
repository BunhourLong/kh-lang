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
