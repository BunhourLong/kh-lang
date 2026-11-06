# T6 — NIT fixes (builder)

Review round 2 returned APPROVE with 5 NITs. Fix them with the smallest diffs.
Allowed edits: `test/**`, `.github/workflows/ci.yml`, `.npmignore`. Nothing else.
Do not change the API, core, fixture data, or dependencies.

1. `test/api.test.mjs:121` — the malformed-shape tests accept any `Error` without a
   code. T5 decisions say these four shapes (`segment("x", 42)`,
   `segment("x", {mode: 5})`, `new Segmenter("x")`, `new Segmenter({dictionary: 42})`)
   throw `ERR_KH_OPTIONS`. Tighten to `assertThrowsCode(..., "ERR_KH_OPTIONS")` for all
   four. If one genuinely throws without that code, stop and report instead of loosening.
2. `test/fuzz.test.mjs:86-89` — `stripBomAndZwsp(text)` is compared to the identical
   `withoutInvisible` expression, so the assertion is vacuous, and line 88 is redundant.
   Remove the duplication; assert something real (e.g. a literal expected value for a
   small ZWSP/BOM case, or drop the redundant assertion) without weakening the fuzz
   invariants.
3. `.github/workflows/ci.yml:31` — package.json and README promise Node >=18, but CI
   tests only Node 20. Change the node matrix to `[18, 20]` (keep the three OSes).
   Verify locally with `npx --yes node@18 --test test/` after a build; it currently
   passes 27/27 under 18.20.8.
4. `.github/workflows/ci.yml` — add a determinism guard after the build/tests:
   `node scripts/generate-gold.mjs && git diff --exit-code -- test/fixtures/gold-100.mjs`.
   Put it where the repo is already built (same job), not in the Rust job.
5. `.npmignore:1` — add `AGENTS.md` and `opencode.json` (real internal files).
   Do not add files that do not exist (there is no `pack.json`); `package.json`,
   README and LICENSE must stay publishable.

## Verification (DONE = all exit 0)

```sh
export PATH="$HOME/.cargo/bin:$PATH"
npm test
npx --yes node@18 --test test/
node scripts/generate-gold.mjs && git diff --exit-code -- test/fixtures/gold-100.mjs
npm pack --dry-run
git diff --check
```

Report only: files changed, last 3 lines per command, and blockers.
