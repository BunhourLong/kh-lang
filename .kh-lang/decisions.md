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
