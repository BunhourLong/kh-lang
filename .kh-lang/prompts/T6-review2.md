# T6 — review round 2 (read-only)

Re-review the T6 change set after the fix round. Ignore `scripts/run-agent.sh`
(separate workflow change, not T6). The T6 change set is:
`git diff -- package.json` plus every untracked file under `test/`, `README.md`,
`LICENSE`, `.npmignore`, `.github/workflows/ci.yml`, and `scripts/generate-gold.mjs`.
Do NOT edit or commit anything.

Sources: `.kh-lang/tasks.md#T6`, `.kh-lang/spec.md`, `.kh-lang/decisions.md`,
`index.d.ts`.

Previous round's BLOCKING findings — verify each is actually fixed:
1. leftover throwing `test/zz-probe.mjs` removed and no non-test helper under `test/`
   is executed by `node --test test/`;
2. fixture generator moved to `scripts/generate-gold.mjs`, no runtime rewrite/side
   effect during tests;
3. `gold-100` rebuilt without filtering by implementation correctness, 100 unique
   dictionary-built sentences, every gold token in `data/khmer.lbdict.txt`,
   `gold.join("") === text`, >=25 bimm/viterbi disagreement sentences, deterministic
   regeneration, and the boundary micro-F1 test still asserts `f1(viterbi) >= f1(bimm)`
   with no self-fulfilling filter;
4. `package.json` `files` includes `licenses`, and the CI pack assertion requires
   `licenses/khmerlbdict-LICENSE`.

Previous NITs — verify: fuzz test now validates malformed input against a
Node-18-compatible well-formed equivalent without hiding invariant failures; README
scopes losslessness to well-formed strings and documents U+FFFD/no-panic; README does
not overclaim prebuilt targets; malformed option shapes are tested with stable error
codes.

Then do a full pass for new issues: test discovery, ESM/Node 18 compatibility, fixture
determinism, F1 math, CI commands on all three OSes, README/API accuracy, `.npmignore`
vs `files`, scope creep, and any remaining BLOCKING/NIT.

Output: numbered findings with `file:line`, labelled BLOCKING or NIT, most severe
first, then a final line exactly `APPROVE` or `REQUEST-CHANGES`. Do not paste full
files or logs.
