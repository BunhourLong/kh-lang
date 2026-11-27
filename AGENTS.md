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

## Workflow (Delikit)
- Project uses the `delikit` skill configured in `.claude/run-task.md`, inheriting global worker routing from `~/.agents/routing.md` via T3 `delegate_task` (never shell launchers or CLI runners).
- `.kh-lang/STATE.md` is the only state that survives `/clear` (one line per task: id, status, commit, blocker); `.kh-lang/decisions.md` is append-only architectural notes.
- One writer per task; trivial/single-file docs or config edits run inline in the main session; run deterministic gates before review; fresh-context reviewer on high-risk tasks (listed in `.claude/run-task.md`) and before release; update `.kh-lang/STATE.md` and stop after one task.

## Rules
- Any task touching 1 file or describable in one sentence: do it inline, skip orchestration.
- Conventional Commits, one concern per commit, never mix fix+feature, never commit
  unverified code. No co-author trailers or AI attribution.
- Deterministic checks (fmt/clippy/tests) run BEFORE review, never instead of it.
- User-visible errors on the JS surface must be structured (NapiError → `code`), not panics.
- Apply `ponytail` (`ultra`) before coding; run `security-audit` on the branch diff before pushing or opening a PR.

Verification commands:
```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm test
npx tsc --noEmit
```
