# T7 — whole-project review before publish (read-only)

Review the whole shipped product: `git diff 37b54dd..HEAD` restricted to product files —
`crates/kh-lang-core/`, `src/`, `Cargo.toml`, `build.rs`, `index.js`, `index.d.ts`,
`package.json`, `.npmignore`, `test/`, `.github/workflows/`, `README.md`, `LICENSE`,
`licenses/`, `scripts/build-dict.mjs`. Skip `data/khmer.lbdict.txt` contents (generated;
check only that it is embedded and attributed), `.kh-lang/`, `scripts/run-agent.sh`, and
agent/workflow config. Do NOT edit any file or commit.

Spec sources: `.kh-lang/spec.md` (acceptance list), `.kh-lang/tasks.md` (T1–T7),
`.kh-lang/decisions.md` (accepted decisions — don't re-litigate them unless they cause a bug).
T1–T6 were each reviewed and verified; look for what per-task reviews miss: cross-module
bugs, inconsistencies between core / napi / types / README, and publish blockers.

Check, in priority order:
1. Correctness: Khmer edge cases (grapheme clusters, coeng U+17D2, ZWSP U+200B, BOM,
   mixed Latin/digits/emoji), losslessness, UTF-16 offsets, bimm/viterbi, custom words.
2. Robustness on the JS surface: no panics/aborts; errors are structured (`code`).
3. Publish readiness: `package.json` (name, version, main/types, files, napi config,
   engines, repository, license), `npm pack` contents, prebuilt binary story
   consistent with README and CI; anything that makes `npm install kh-lang` fail.
4. Licensing/attribution of the SIL dictionary.
5. Dead code, duplicated logic, needless complexity (ponytail): name what to delete.

Output: numbered findings with `file:line`, each labelled BLOCKING or NIT, most severe
first, then a final line exactly `APPROVE` or `REQUEST-CHANGES`. Do not paste full
files or logs.
