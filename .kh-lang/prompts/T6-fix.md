# T6 — fix round (builder)

Fix the T6 review findings below. Same repo and same task scope; T5 API/core stay
untouched. Allowed edits: `test/**`, `scripts/generate-gold.mjs` (new, moved out of
test discovery), `README.md`, `LICENSE`, `.npmignore`, `.github/workflows/ci.yml`,
`package.json` (only `pretest` and `files`), `package-lock.json` only if npm changes it.
Do not edit `crates/`, `src/`, `index.js`, `index.d.ts`, `Cargo.*`, `tsconfig.json`,
`data/`, or `.kh-lang/`. No new dependencies.

## BLOCKING (must fix)

1. `test/zz-probe.mjs` is a leftover probe that throws at import; delete it. Ensure no
   non-test helper/script lives under `test/` in a way `node --test test/` executes.
2. `test/fixtures/generate-gold.mjs` runs during `npm test`, rewriting the fixture while
   the test imports it. Move the generator to `scripts/generate-gold.mjs` (keep it
   runnable for provenance) and keep only static data under `test/fixtures/`.
3. `test/fixtures/generate-gold.mjs` currently keeps only sentences that BOTH modes
   already segment exactly, so F1 is 1.0/1.0 and the comparison proves nothing.
   Rebuild `test/fixtures/gold-100.mjs` as exactly 100 pure-Khmer sentences whose gold
   boundaries are known independently of the implementation:
   - Compose each sentence by concatenating 2–6 distinct embedded-dictionary headwords;
     gold = those headwords. Every gold token must be present in
     `data/khmer.lbdict.txt`; verify `gold.join("") === text`.
   - Do NOT filter candidates by whether bimm or viterbi matches gold.
   - The sample MUST include mode-disagreement cases: at least 25 of the 100 sentences
     where bimm and viterbi produce different token boundaries. Sampling disagreements
     is allowed; sampling by which mode wins is not.
   - No duplicate sentences. Use a fixed seed so regeneration is deterministic.
   - Record provenance in a fixture comment: constructed from dictionary headwords,
     every gold token dictionary-verified, disagreements included.
   - Keep the boundary micro-F1 test asserting `f1(viterbi) >= f1(bimm)` over all 100.
     If that fails, stop and report the numbers; do not tune the fixture to make it pass.
4. `package.json` `files` omits `licenses/`, but README links to the SIL notice and the
   published tarball must carry it. Add `"licenses"` to `files` and assert the pack job
   includes `licenses/khmerlbdict-LICENSE`.

## NITs (fix unless a concrete reason not to)

5. `test/fuzz.test.mjs` broad catch hides token-invariant failures. For malformed UTF-16,
   assert no abort AND validate tokens against a well-formed equivalent computed in the
   test (implement a Node-18-compatible `wellFormed()` helper in `test/helpers.mjs`;
   do NOT use `String.prototype.toWellFormed`, engines is >=18). Valid strings keep the
   strict `join("") === input` assertion; lone surrogates are expected to become U+FFFD.
6. README says segmentation "always reproduces the input exactly". Scope that claim to
   well-formed strings and document that lone/unpaired UTF-16 surrogates are replaced
   with U+FFFD by the N-API string conversion (no panic). Keep spec #4's no-panic
   guarantee explicit.
7. README claims five prebuilt targets while CI only host-builds three OSes. Either add
   the remaining targets to CI (hard) or soften the wording: list the five configured
   napi targets separately from the three CI-verified host builds; do not imply the
   others are published/verified.
8. `test/api.test.mjs` "invalid modes and options" tests only mode values. Add malformed
   option shapes (`segment("x", 42)`, `segment("x", {mode: 5})`, `new Segmenter("x")`,
   `new Segmenter({dictionary: 42})`) and assert `ERR_KH_OPTIONS` or a normal Error as
   the T5 decisions allow; no process abort.

## Verification (DONE = all exit 0)

```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
rm -f ./*.node
npm ci
npm test
npx tsc --noEmit
npm pack --dry-run
git diff --check
```

Also run `node scripts/generate-gold.mjs` once and confirm it reproduces
`test/fixtures/gold-100.mjs` byte-for-byte (`git diff --exit-code -- test/fixtures`),
and confirm `node --test test/` does not execute the generator. Report only: files
changed, last 3 lines per command, F1 numbers for both modes, disagreement count, and
blockers. If the brief is wrong or ambiguous, stop and report with evidence.
