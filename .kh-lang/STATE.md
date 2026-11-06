# STATE.md — the only state that survives /clear
| task | status | commit | blocker |
|------|--------|--------|---------|
| T1 data | done | 462aaaa | — |
| T2 core-foundation | done | 6027859 | — |
| T3 core-matchers | done | 73ab4a2 | — |
| T4 core-segmenter | open | | next: run in a fresh session |
| T5 napi-glue | open | | depends on T4; adds root Cargo.toml/package.json |
| T6 tests+ci+docs | open | | depends on T5 |
| T7 review+publish | open | | after T6 |

Verified at T3: 36 unit tests + 5 doctests green; fmt/clippy clean.
Reviewer findings addressed in 73ab4a2 (see decisions.md).
Next session: read this file, then dispatch T4 briefly (builder) via
`bash scripts/run-agent.sh builder .kh-lang/prompts/T4.md`.
