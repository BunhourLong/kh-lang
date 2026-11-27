# Task configuration

- Plan and next task: .kh-lang/STATE.md and .kh-lang/tasks.md
- Notes: .kh-lang/decisions.md
- Gates: export PATH="$HOME/.cargo/bin:$PATH" && cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && npm test && npx tsc --noEmit
- Focused test: cargo test -p kh-lang-core <test_name> | node --test test/<file>.test.mjs
- Constraints: CLAUDE.md, AGENTS.md (including banned patterns + ban-on-sight with deterministic lint/clippy/test enforcement), .kh-lang/spec.md, .kh-lang/decisions.md; crates/kh-lang-core stays zero-dep & pure Rust (#![deny(missing_docs)]); lossless segmentation; structured N-API error codes (ERR_KH_OPTIONS, ERR_KH_WORDS); apply ponytail (ultra) before coding
- High risk: changes to crates/kh-lang-core/src/{cluster,matcher,segmenter}.rs, src/index.rs (FFI/unsafe), .github/workflows/*, release/publish scripts
- Review: coding-phase review via fresh-context reviewer per ~/.agents/routing.md on high-risk tasks and before release; pre-push / rollout pen-test via `security-audit` skill (plus `github-actions-hardening` for workflows) on `claudeAgent_2` / `claude-opus-5-5` (`effort: high`)
- Git: Conventional Commits, one concern per commit, no AI attribution; never stage unrelated changes; commit locally only and never push/tag until the pre-push `security-audit` pen-test passes AND the user explicitly authorizes pushing
- Close: update .kh-lang/STATE.md (and .kh-lang/decisions.md for architectural choices); stop after one task

## Worker routing

Inherits `~/.agents/routing.md` (and `~/.agents/AGENTS.md`). Project-specific overrides:

| Job | Provider and model | Options |
| --- | --- | --- |
| Pre-push / rollout security pen-test (`security-audit`) | `claudeAgent_2` / `claude-opus-5-5` | `effort: high` |
