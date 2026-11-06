# STATE.md — the only state that survives /clear
| task | status | commit | blocker |
|------|--------|--------|---------|
| T1 data | done | 462aaaa | — |
| T2 core-foundation | done | 6027859 | — |
| T3 core-matchers | done | 73ab4a2 | — |
| T4 core-segmenter | done | 1ce81e1 | — |
| T5 napi-glue | done | 69fc69c | — |
| T6 tests+ci+docs | done | 68dbb73 | — |
| T7 review+publish | pushed; publish pending | 8ba6a3c | NPM_TOKEN repo secret, then tag v0.1.0 |

Verified at T4: 50 unit + 9 doctests green; fmt/clippy clean; independent tester
losslessness checks on Khmer, mixed Latin/digits/emoji, and ZWSP inputs passed.
Reviewer APPROVE; its 7 NITs were fixed before 1ce81e1 (see decisions.md).

Verified at T6: tester ALL PASS — fmt/clippy/50 unit + 9 doctests green; clean-clone
`npm ci && npm test` 25/25; Node 18 27/27; `tsc --noEmit`; generator determinism guard;
`npm pack` 7 files including `licenses/` and no internal paths; F1 bimm 0.9439 vs
viterbi 0.9847. Reviewer APPROVE (0 BLOCKING) after one fix round; 5 NITs fixed.

Workflow: `run-agent.sh` now prints the brief, titles the session, streams tool
activity, and prints `opencode --session <id>` so jobs are inspectable in OpenCode.
Verified at T7 (subagents on Sonnet 5.5 via Claude Code; opencode's `steps` cap sends an
assistant prefill that Sonnet 5.5 rejects): whole-project review → 2 fix rounds (user
approved the 2nd) → final APPROVE. Tester ALL PASS: fmt/clippy/tests, npm test 25/25, tsc,
pack has no `.node`, `npm ci` on unchanged lockfile, gold determinism, and a packed
root + darwin-arm64 platform package install that segments losslessly. Binaries ship as napi
platform packages (`npm/`), published by `.github/workflows/release.yml` on tag `v*`.
Remaining: add the `NPM_TOKEN` secret, then `git tag v0.1.0 && git push origin v0.1.0` and
watch the release run. The release workflow has not run yet, so runner labels are unverified.
