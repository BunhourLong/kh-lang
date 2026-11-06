# T7 — fix round 2 (user-approved)

The uncommitted T7-fix changes are in the working tree. The review (`.kh-lang/prompts/T7-review2.md`)
returned REQUEST-CHANGES. Apply exactly these fixes and nothing else. The original brief is
`.kh-lang/prompts/T7-fix.md`.

## BLOCKING
1. **Remove `optionalDependencies` from the root `package.json`.** napi-rs 3.10.12 `prepublish`
   builds the root optionalDependencies at publish time (`node_modules/@napi-rs/cli/dist/index.js`
   ~15444–15530), so declaring them breaks `npm ci`: the lockfile doesn't have them and the
   packages aren't on npm yet.
   - Leave `package-lock.json` unchanged.
   - In `.github/workflows/ci.yml`, remove the pack-check assertion about optionalDependencies.
     Keep the root `.node` assertion.
   - Verify that `npm ci --ignore-scripts` succeeds in a scratch copy that has only
     `package.json` and `package-lock.json`.
2. **`.github/workflows/release.yml`.** Replace the retired `macos-13` runner with
   `macos-15-intel` for x86_64-apple-darwin.
3. **`npm/*/package.json` (all 5).** Regenerate with `npx napi create-npm-dirs` now that the
   root has the metadata, so each one carries `repository`, `homepage`, `bugs`, `keywords`,
   `publishConfig`, `license` and `engines` from the root.
   - Keep the `os`/`cpu`/`libc` values that are already correct.
   - If regenerating doesn't carry the fields, add them by hand to match the root.

## NIT
4. `package.json` `prepublishOnly`: change it to `napi prepublish -t npm --no-gh-release`.
   - Verify the flag parses: `npx napi prepublish --help`, or run a dry invocation that
     exits on a parse error, not on the network.
5. `release.yml`: add a first step in the publish job, or a guard job that `build` needs,
   that fails unless `${GITHUB_REF_NAME#v}` equals `package.json` `version`.
6. `release.yml` smoke test: build the Khmer input from `\u` escapes inside the JS
   (`'ខ្ញុំ'`), not a raw shell literal. Also assert that
   `segment(s).length >= 1` and `s.length > 0`, so the check is not vacuous.

## Allowed files
`package.json`, `npm/**`, `.github/workflows/ci.yml`, `.github/workflows/release.yml`. Nothing else.
Do not commit, tag, push or publish.

## DONE means all of these exit 0
```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm test
npm pack --dry-run --json | node -e 'const p=JSON.parse(require("fs").readFileSync(0))[0];if(p.files.some(f=>f.path.endsWith(".node")))process.exit(1)'
d=$(mktemp -d) && cp package.json package-lock.json "$d" && (cd "$d" && npm ci --ignore-scripts --no-audit --no-fund) && rm -rf "$d"
ruby -e 'require "yaml"; ARGV.each{|f| YAML.load_file(f)}' .github/workflows/*.yml
for p in npm/*/package.json; do node -e 'const p=require("./'"$p"'");for(const k of ["repository","publishConfig","license","os","cpu"]) if(!p[k]) {console.error(k);process.exit(1)}'; done
```
Report only the files you changed and the last lines of each command's output.
