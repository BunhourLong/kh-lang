# T6 — review (read-only)

Review ONLY the uncommitted T6 changes in this repo. The change set is:
`git diff` (package.json only) plus every file under `test/`, `README.md`,
`LICENSE`, `.npmignore`, and `.github/workflows/ci.yml` (all untracked).
Do NOT review or modify anything else, and do NOT edit any file or commit.

Spec sources: `.kh-lang/tasks.md` section `T6`, `.kh-lang/spec.md` acceptance list,
`.kh-lang/decisions.md` (T5 API semantics), `index.d.ts` (fixed public surface).
T5 is verified; the builder reported one accepted limitation: N-API replaces lone
UTF-16 surrogates with U+FFFD, so strict losslessness is asserted only for well-formed
strings; `segment("\uD800")` must not panic. Judge whether that handling satisfies
spec #4.

Check, in priority order:
1. Every testable acceptance criterion (#1, #2, #3 F1, #4 no-panic, #5 custom words
   and chaining, #6 API/types) is genuinely covered, with real assertions — no
   tautologies, no tests that pass vacuously, no skipped/todo tests.
2. `test/fixtures/gold-100.mjs`: exactly 100 entries, each `gold.join("") === text`,
   `gold.length >= 2`, every gold token present in `data/khmer.lbdict.txt`, no duplicated
   sentence, and F1 is computed correctly (boundary micro-F1, viterbi >= bimm).
   Flag any fixture curation that makes the comparison meaningless.
3. Losslessness/UTF-16 offsets, emoji surrogate pairs, ZWSP/BOM, coeng, mixed runs,
   invalid options/words error codes, no process abort.
4. CI correctness: fmt → clippy → test → native build matrix → `npm test`;
   self-contained `npm test` from no prebuilt `.node`; `npm pack --dry-run` assertion
   really checks tarball contents; workflow syntax/commands valid on all three OSes.
5. Docs accuracy vs the real API: README install/usage/API/modes/custom words/build,
   Khmer section is real Khmer, license/attribution correct; `.npmignore` consistent
   with package.json `files`; no overclaiming.
6. Scope creep: only the allowed files, no new dependencies, no API/core changes.

Output: numbered findings with `file:line`, each labelled BLOCKING or NIT, most severe
first, then a final line exactly `APPROVE` or `REQUEST-CHANGES`. Do not paste full
files or logs.
