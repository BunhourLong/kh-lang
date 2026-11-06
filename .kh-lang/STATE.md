# STATE.md — the only state that survives /clear
| task | status | commit | blocker |
|------|--------|--------|---------|
| T1 data | done | 462aaaa | — |
| T2 core-foundation | done | 6027859 | — |
| T3 core-matchers | done | 73ab4a2 | — |
| T4 core-segmenter | done | 1ce81e1 | — |
| T5 napi-glue | done | 69fc69c | — |
| T6 tests+ci+docs | done | 68dbb73 | — |
| T7 review+publish | open | | after T6 |

Verified at T4: 50 unit + 9 doctests green; fmt/clippy clean; independent tester
losslessness checks on Khmer, mixed Latin/digits/emoji, and ZWSP inputs passed.
Reviewer APPROVE; its 7 NITs were fixed before 1ce81e1 (see decisions.md).

Verified at T6: tester ALL PASS — fmt/clippy/50 unit + 9 doctests green; clean-clone
`npm ci && npm test` 25/25; Node 18 27/27; `tsc --noEmit`; generator determinism guard;
`npm pack` 7 files including `licenses/` and no internal paths; F1 bimm 0.9439 vs
viterbi 0.9847. Reviewer APPROVE (0 BLOCKING) after one fix round; 5 NITs fixed.

Workflow: `run-agent.sh` now prints the brief, titles the session, streams tool
activity, and prints `opencode --session <id>` so jobs are inspectable in OpenCode.
Task T6 is complete. STOP here; start T7 in a NEW session, rehydrate from this file
only, write `.kh-lang/prompts/T7.md`, then dispatch the reviewer.
