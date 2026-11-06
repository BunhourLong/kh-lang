# AGENTS.md — kh-lang

Khmer text segmentation library. Pure-Rust core (`crates/kh-lang-core`) with Node.js
bindings (`src/index.rs`, napi-rs) published to npm as `kh-lang`. Global standards in
`~/.config/opencode/AGENTS.md` apply.

## Stack & layout
- Rust core crate `crates/kh-lang-core`: pure, no napi/wasm types, no external deps.
- napi glue crate at repo root (`Cargo.toml` + `src/index.rs`): translation only.
- `data/khmer.lbdict.txt`: embedded dictionary (SIL khmerlbdict, MIT — see `licenses/`).
- `scripts/build-dict.mjs`: rebuilds the dictionary from `scripts/data-source/`.
- Toolchain: `export PATH="$HOME/.cargo/bin:$PATH"` (rustup stable).

## Workflow (orchestrator → subagents)
The main session is an **orchestrator**: it never reads code or test logs; it decomposes,
dispatches, and verifies artifacts. All shared state lives in `.kh-lang/` files — pass
PATHS, not contents. `.kh-lang/STATE.md` is the only state that survives `/clear`; one
line per task: id, status, commit, blocker.

Per task:
1. Dispatch a **builder** with `.kh-lang/tasks.md#<ID>` as the brief; allow-list its
   files; it must stop and report if the brief is ambiguous, and DONE = the task's
   exact verification commands exit 0.
2. Dispatch a **reviewer** (read-only) on the diff: BLOCKING vs NIT findings with
   `file:line`; loop back once max, else STOP and ask the user. **tester** runs the
   final verification commands.
3. Update `.kh-lang/STATE.md` → commit → after the task is complete, `/clear` the
   orchestrator and rehydrate from STATE.md only.

Dispatch is VISIBLE: write the brief to `.kh-lang/prompts/<ID>.md`, then run
`bash scripts/run-agent.sh <builder|reviewer|tester|scout> .kh-lang/prompts/<ID>.md`.
A cmux split opens beside the orchestrator and streams the live session; each job gets
a fresh context and runs `opencode/deepseek-v4.1-flash` with `--variant max`
(override with `AI_MODEL` / `AI_VARIANT`). Outside cmux it still runs, unseen.
Never run two jobs that edit the same files at once.

Templates:
```
Implementer: Task {ID} — {desc}. Read: .kh-lang/tasks.md#{ID}, plus only: {paths ≤6}.
No changes outside {allowed}. Push back with evidence if the brief is wrong.
DONE means: {exact commands}. Report: files changed + last lines of test output only.

Reviewer: Review ONLY {diff/range} against .kh-lang/tasks.md#{ID}. Spec compliance,
Khmer edge cases (grapheme clusters, coeng U+17D2, ZWSP U+200B, mixed Latin/digits),
scope creep, error handling. Output: numbered BLOCKING/NIT findings with file:line,
then APPROVE or REQUEST-CHANGES.
```

## Rules
- Any task touching 1 file or describable in one sentence: do it inline, skip orchestration.
- Conventional Commits, one concern per commit, never mix fix+feature, never commit
  unverified code. No co-author trailers.
- Deterministic checks (fmt/clippy/tests) run BEFORE review, never instead of it.
- User-visible errors on the JS surface must be structured (NapiError → `code`), not panics.

Verification commands:
```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
node build-check.mjs   # quick sanity build of the napi import (or `npm test` once wired)
```
