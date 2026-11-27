# Task configuration

- Plan and next task: .kh-lang/STATE.md and .kh-lang/tasks.md
- Notes: .kh-lang/decisions.md
- Gates: export PATH="$HOME/.cargo/bin:$PATH" && cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace && npm test && npx tsc --noEmit
- Focused test: cargo test -p kh-lang-core <test_name> | node --test test/<file>.test.mjs
- Constraints: AGENTS.md, .kh-lang/spec.md, .kh-lang/decisions.md; crates/kh-lang-core stays zero-dep & pure Rust (#![deny(missing_docs)]); lossless segmentation; structured N-API error codes (ERR_KH_OPTIONS, ERR_KH_WORDS); apply ponytail (ultra) before coding
- High risk: changes to crates/kh-lang-core/src/{cluster,matcher,segmenter}.rs, src/index.rs (FFI/unsafe), .github/workflows/*, release/publish scripts
- Review: fresh-context reviewer per ~/.agents/routing.md on high-risk tasks and before release
- Git: Conventional Commits, one concern per commit, no AI attribution; never stage unrelated changes; run security-audit on branch diff before push/PR; push/tag only when user authorizes
- Close: update .kh-lang/STATE.md (and .kh-lang/decisions.md for architectural choices); stop after one task

## Worker routing

Inherits `~/.agents/routing.md` (and `~/.agents/AGENTS.md`). No project-specific overrides.
