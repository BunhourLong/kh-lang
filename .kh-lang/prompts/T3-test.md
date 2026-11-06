# T3 verification (tester)

Repo: /Users/capon/Documents/Projects/kh-lang
Work in `crates/kh-lang-core`. Do not edit files.

Run exactly:
1. `export PATH="$HOME/.cargo/bin:$PATH"; cargo fmt --all --check; echo EXIT=$?`
2. `cargo clippy --all-targets -- -D warnings; echo EXIT=$?`
3. `cargo test; echo EXIT=$?`
4. `cargo test --doc; echo EXIT=$?`

Then report ONLY: command, exit code, and the `test result:` line for 3 and 4.
End with OVERALL: PASS if all exit 0, else FAIL. Under 100 words.
