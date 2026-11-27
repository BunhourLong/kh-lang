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

## Banned patterns (codebase-as-memory)
The codebase is the memory future AI sessions imitate. When a bad pattern, footgun, or recurring workaround is found, do not stack workarounds — fix the root cause, enforce a deterministic lint/clippy/test check where possible, and add the pattern here (imported by `CLAUDE.md`) so it is permanently banned:
- `unwrap()`, `expect()`, `panic!()`, or unchecked indexing in non-test Rust code (surface structured `ERR_KH_*` N-API errors instead of panicking across FFI).
- Deep-cloning the 33,895-entry embedded dictionary in new code (use `&'static Dictionary` or a lightweight overlay; `T10` removes the legacy clone in `src/index.rs`).
- External crates or `napi`/`wasm` types inside `crates/kh-lang-core`.
- Mutable action tags/branches (`@v4`, `@main`, `@stable`), missing `permissions: {}` or `persist-credentials: false`, or `npm ci` / `npm publish` without `--ignore-scripts` in `.github/workflows/`.
- JS-length-driven preallocation (`Vec::with_capacity(js_len)`) or prototype-chain option reads (`has_own_property` required) in `src/index.rs`.
- Shell-launcher scripts or CLI subagent runners (use Delikit + T3 `delegate_task` only).

## Rules
- Any task touching 1 file or describable in one sentence: do it inline, skip orchestration.
- Conventional Commits, one concern per commit, never mix fix+feature, never commit
  unverified code. No co-author trailers or AI attribution.
- Never push commits to `main` (or any remote) until everything is committed, verified, security-audited, AND the user explicitly asks to push.
- Deterministic checks (fmt/clippy/tests) run BEFORE review, never instead of it.
- Two-phase review: during coding, run deterministic gates (`fmt`/`clippy`/`cargo test`/`npm test`/`tsc`) + Delikit code review per `.claude/run-task.md`; before pushing to `main` or rolling out a release, run a fresh-context pen-test using the `security-audit` skill (plus `github-actions-hardening` if workflows changed) on `claudeAgent_2` / `claude-opus-5-5` (`effort: high`), and fix or report every confirmed finding before pushing.
- User-visible errors on the JS surface must be structured (NapiError → `code`), not panics.
- Apply `ponytail` (`ultra`) before coding.

Verification commands:
```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm test
npx tsc --noEmit
```
