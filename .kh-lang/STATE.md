# STATE.md — the only state that survives /clear
| task | status | commit | blocker |
|------|--------|--------|---------|
| T1 data | done | 462aaaa | — |
| T2 core-foundation | done | 6027859 | — |
| T3 core-matchers | done | 73ab4a2 | — |
| T4 core-segmenter | done | 1ce81e1 | — |
| T5 napi-glue | done | 69fc69c | — |
| T6 tests+ci+docs | done | 68dbb73 | — |
| T7 review+publish | v0.1.0 ready to rerun (win32 spam block cleared; CI/release hardened, Delikit refreshed) | 2cf041a | finish v0.1.0 release & approve staged root `kh-lang` |
| T8 cluster-marks-and-js-validation | todo | — | queued for v0.1.1 after v0.1.0 release |

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

2026-11-11 update: 4 platform pkgs approved & live (darwin-arm64/x64, linux-x64/arm64-gnu @0.1.0).
Rerun of 42236039161 failed differently: publish is not idempotent — `napi prepublish`
re-published already-live platform pkgs and died on kh-lang-darwin-x64 E403 before reaching
win32/root. Fix: release.yml publish job made idempotent (skip `npm view`-published pkgs,
root optionalDependencies wired manually + `--ignore-scripts`; version-match trust boundary
in the loop). Impl glm-5.3-flash PASS, reviewer deepseek-v4.1-flash PASS. Committed 2cf041a;
v0.1.0 tag force-moved to it; new run 43017693366. Context windows on opencode-go for both
glm-5.3-flash and deepseek-v4.1-flash are 1M tokens — no subagent context cap needed.
2026-11-27 update: npm support cleared `kh-lang-win32-x64-msvc` (transferred to
`bunhourlong`, currently holding `0.0.1-security`; `0.1.0` returns 404). Workflow refreshed
to Delikit (`.claude/run-task.md`) and `.github/workflows/{ci,release}.yml` hardened with
zero Rust/JS runtime changes. Next: move tag `v0.1.0` to the hardened release commit and
push so `Release` runs fresh `verify` + 5-target `build`, skips the 4 live `0.1.0`
packages, and publishes `kh-lang-win32-x64-msvc@0.1.0` then root `kh-lang@0.1.0` (if
staged at `0.0.0-stage`, approve Windows first, then root `kh-lang` with 2FA on npmjs.com).
