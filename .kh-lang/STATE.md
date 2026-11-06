# STATE.md — the only state that survives /clear
| task | status | commit | blocker |
|------|--------|--------|---------|
| T1 data | done | 462aaaa | — |
| T2 core-foundation | done | 6027859 | — |
| T3 core-matchers | done | 73ab4a2 | — |
| T4 core-segmenter | done | 1ce81e1 | — |
| T5 napi-glue | done | 69fc69c | — |
| T6 tests+ci+docs | done | 68dbb73 | — |
| T7 review+publish | v0.1.0 tagged; publish partial | 2cdd1cb | npm spam block on kh-lang-win32-x64-msvc; approve 4 staged pkgs; rerun publish |

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
Post-review, CI fixes: Node floor raised to >=20 (napi CLI needs util.styleText), and
`node --test` default discovery (Node 22+ rejects `node --test test/`). CI is all green.
Release run 42236039161: all 5 builds and smoke tests passed. In publish, the darwin-x64,
darwin-arm64, linux-x64-gnu and linux-arm64-gnu 0.1.0 releases went into npm's staged queue,
since a new package's first version is always staged; the public version reads `0.0.0-stage`.
kh-lang-win32-x64-msvc got E403 "Package name triggered spam detection", so root `kh-lang`
was never published.
Next steps:
1. The user asks npm support to clear the spam block.
2. The user approves the 4 staged packages (npmjs.com → Staged Packages, with 2FA).
3. `gh run rerun 42236039161 --failed`.
4. Approve the staged win32 and root `kh-lang`.
