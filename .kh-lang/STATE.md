# STATE.md — the only state that survives /clear
| task | status | commit | blocker |
|------|--------|--------|---------|
| T1 data | done | 462aaaa | — |
| T2 core-foundation | done | 6027859 | — |
| T3 core-matchers | done | 73ab4a2 | — |
| T4 core-segmenter | done | 1ce81e1 | — |
| T5 napi-glue | done | 69fc69c | — |
| T6 tests+ci+docs | open | | depends on T5 |
| T7 review+publish | open | | after T6 |

Verified at T4: 50 unit + 9 doctests green; fmt/clippy clean; independent tester
losslessness checks on Khmer, mixed Latin/digits/emoji, and ZWSP inputs passed.
Reviewer APPROVE; its 7 NITs were fixed before 1ce81e1 (see decisions.md).

Verified at T5: independent tester ALL PASS — fmt/clippy/no warnings, 50 unit + 9
doctests green, `npm ci`, `napi build --platform --no-const-enum`, `tsc --noEmit`,
and 16/16 JS acceptance assertions (lossless bimm/viterbi, UTF-16 offsets, custom
words, error codes, lone surrogate survives). Reviewer APPROVE after one fix round
(JS-length allocation, normalize docs, prototype-pollution guard); remaining NITs in
decisions.md.

Workflow: `run-agent.sh` now passes the variant as `model#variant` (8e7aa6f).
Task T5 is complete. STOP here; start T6 in a NEW session, rehydrate from this file
only, write `.kh-lang/prompts/T6.md`, then dispatch the builder.
