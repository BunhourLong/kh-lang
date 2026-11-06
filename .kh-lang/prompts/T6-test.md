# T6 — final verification (tester, no edits)

Repo: /Users/capon/Documents/Projects/kh-lang
Read `.kh-lang/prompts/T6.md`, `.kh-lang/prompts/T6-fix.md`, and
`.kh-lang/prompts/T6-nit.md` for the intended deliverables. Do NOT edit any file and
do NOT commit. Run everything from the repo root with
`export PATH="$HOME/.cargo/bin:$PATH"`.

## Deterministic checks

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
bash -n scripts/run-agent.sh
git diff --check
```

## Clean-clone Node path

```sh
rm -f ./*.node
npm ci
npm test
npx --yes node@18 --test test/
npx tsc --noEmit
node scripts/generate-gold.mjs
git diff --exit-code -- test/fixtures/gold-100.mjs
npm pack --dry-run
```

Additionally assert, with shell/node assertions (not eyeballing):
1. `test/` contains no executable generator or probe; `node --test test/` does not
   rewrite `test/fixtures/gold-100.mjs` (compare mtime/hash before/after).
2. `npm pack --dry-run` / `npm pack --json --dry-run` includes `index.js`, `index.d.ts`,
   `README.md`, `LICENSE`, `licenses/khmerlbdict-LICENSE`, `package.json`, and a
   `*.node` binary; excludes `test/`, `.kh-lang/`, `target/`, `node_modules/`,
   `AGENTS.md`, `opencode.json`.
3. `test/fixtures/gold-100.mjs` has exactly 100 unique sentences, each
   `gold.join("") === text`, `gold.length >= 2`, and at least 25 bimm/viterbi
   disagreements. The gold test reports F1 for both modes and viterbi >= bimm.
4. `node -e` smoke: `segment("ខ្ញុំស្រលាញ់ភ្នំពេញ")` equals the 3 expected tokens,
   `segment(s).join("") === s` for Khmer/Latin/digits/emoji/ZWSP input in both modes,
   `normalize("ក\u200Bខ") === "កខ"`, `insertZwsp` round-trip, `addWords` chaining,
   stable error codes, and `segment("\uD800")` does not abort the process.
5. `git status --short` contains only T6 deliverables (`.github/`, `.npmignore`,
   `README.md`, `LICENSE`, `test/`, `scripts/generate-gold.mjs`, `package.json`),
   workflow changes (`scripts/run-agent.sh`, `.opencode/agents/*.md`,
   `.kh-lang/prompts/T6*.md`), and the state file; no changes under `crates/`, `src/`,
   `data/`, `index.js`, `index.d.ts`, or `Cargo.*`.

Report a compact table: check, PASS/FAIL, and for failures the exact command, exit
code, and last 5 lines. End with `ALL PASS` or `FAILURES: <n>`. Do not paste full logs.
