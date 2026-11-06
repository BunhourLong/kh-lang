# T7 — fix: ship prebuilt binaries as napi platform packages

Context: `.kh-lang/tasks.md#T7`. The whole-project review returned REQUEST-CHANGES. The user
chose the standard napi-rs v3 layout (installed `@napi-rs/cli` 3.10.12): per-platform npm
packages plus `optionalDependencies`, published from CI on a version tag. The root package
ships no `.node` file. Read the CLI's own help (`npx napi --help`, `npx napi <cmd> --help`)
instead of guessing flags.

## Required changes
1. **Platform packages.** Generate `npm/<platform>/package.json` (+ README) for the five
   `napi.targets` with `napi create-npm-dirs`. Remove `npm/` from `.gitignore` so they are
   committed. Each sub-package's `main` must match the filename that `index.js` looks for.
2. **Root package.json.**
   - Remove `*.node` from `files`.
   - Add `optionalDependencies` for all five `kh-lang-<platform>` packages, pinned to `0.1.0`.
   - Add `repository` (git+https://github.com/BunhourLong/kh-lang.git), `homepage`, `bugs`,
     `keywords` (khmer, segmentation, word-break, tokenizer, napi), and `"publishConfig": {"access": "public"}`.
   - Add scripts in the napi v3 template form: `artifacts` (`napi artifacts`),
     `prepublishOnly` (`napi prepublish -t npm`, plus whatever flag stops it creating a GitHub
     release; check `--help`), and `version` (`napi version`).
   - Do not change `name`, `version`, `main`, `types`, `engines` or `napi`.
3. **Release workflow `.github/workflows/release.yml`.**
   - Trigger: push of tags `v*`.
   - Build matrix covering all five targets on native runners where possible:
     macos-latest for aarch64-apple-darwin, macos-13 for x86_64-apple-darwin,
     ubuntu-latest for x86_64-unknown-linux-gnu, windows-latest for x86_64-pc-windows-msvc.
     For aarch64-unknown-linux-gnu, use ubuntu-24.04-arm (native) or the CLI's
     cross-compilation flag.
   - Each build runs `npm ci`, then `npx napi build --platform --release --no-const-enum --target <t>`,
     and uploads the `.node` file as an artifact.
   - Smoke-test each binary on runners that can load it natively: `node -e` requiring `./index.js`
     and asserting `segment('ខ្ញុំស្រលាញ់ភ្នំពេញ').join('') === input`.
   - A publish job (`needs: build`, ubuntu-latest, Node 20, `registry-url: https://registry.npmjs.org`)
     downloads every artifact and runs `npm run artifacts`. It then runs `npm publish --access public`
     with `NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}`; prepublishOnly publishes the sub-packages first.
     Use `permissions: contents: read` plus `id-token: write` and `--provenance`.
4. **CI `.github/workflows/ci.yml` pack job.** Stop duplicating the node matrix (reviewer NIT 5):
   keep only checkout, setup, rust, `npm ci`, build, and pack.
   - Build with `npm run build` (release mode).
   - Keep the generate-gold determinism guard. If it needs the binary, keep the build step before it.
   - Change the pack assertion to **fail if any `.node` file is in the root tarball**. Also assert
     that `package.json` lists every `npm/*` package name in `optionalDependencies`.
5. **Cleanup NITs.**
   - `git rm crates/kh-lang-core/Cargo.lock`. The root `Cargo.lock` is tracked and authoritative.
   - Delete `.npmignore`, because `files` is authoritative.
   - Add `rust-version = "1.85"` to both Cargo.toml `[package]` sections (edition 2024).
6. **README install section (README.md:10-22).** State accurately that prebuilt binaries ship for
   the five targets through platform packages, which npm installs automatically. Everything else
   needs [Build from source]. Make no other README changes.

## Allowed files
`package.json`, `package-lock.json` (only if npm rewrites it), `.gitignore`, `npm/**`,
`.github/workflows/ci.yml`, `.github/workflows/release.yml`, `Cargo.toml`,
`crates/kh-lang-core/Cargo.toml`, `crates/kh-lang-core/Cargo.lock` (delete), `.npmignore` (delete),
`README.md`. Nothing else. Do not touch `src/`, `crates/*/src`, `index.js`, `index.d.ts` or `test/`.
Do not commit, publish, tag or push.

If the brief conflicts with how napi-rs 3.10.12 actually works, stop and report with evidence.

## DONE means all of these exit 0
```sh
export PATH="$HOME/.cargo/bin:$PATH"
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm test
npm pack --dry-run --json | node -e 'const p=JSON.parse(require("fs").readFileSync(0))[0];if(p.files.some(f=>f.path.endsWith(".node")))process.exit(1)'
node -e 'for (const f of [".github/workflows/ci.yml",".github/workflows/release.yml"]) require("fs").readFileSync(f,"utf8")' && npx --yes yaml-lint .github/workflows/*.yml || ruby -e 'require "yaml"; ARGV.each{|f| YAML.load_file(f)}' .github/workflows/*.yml
```
Report only the files you changed and the last lines of each command's output.
