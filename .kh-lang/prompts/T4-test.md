# T4 verification (tester)

Repo: /Users/capon/Documents/Projects/kh-lang
Work in `crates/kh-lang-core`. Do not edit repo files.

Run exactly, with `export PATH="$HOME/.cargo/bin:$PATH"`:
1. `cargo fmt --all --check; echo EXIT=$?`
2. `cargo clippy --all-targets -- -D warnings; echo EXIT=$?`
3. `cargo test; echo EXIT=$?`
4. `cargo test --doc; echo EXIT=$?`

Then make 3 **independent** losslessness assertions outside the repo test suite:
create a throwaway cargo project under `$(mktemp -d)` with a path dependency on
`/Users/capon/Documents/Projects/kh-lang/crates/kh-lang-core`, and assert:
- `concat(Segmenter::new().segment(s)) == s` for
  `s = "ខ្ញុំស្រលាញ់ភ្នំពេញ"`
- same for `s = "Hello 123, ខ្មែរ! 45.6% 🙂"`
- same for `s = "ខ្មែរ\u{200B}ភាសា test\u{200B}99"`
Also assert `Segmenter::new().insert_zwsp("ខ្ញុំស្រលាញ់ភ្នំពេញ").replace('\u{200B}', "")`
equals the input. Print `OK <label>` per assertion; clean up the temp dir.

Report ONLY: command, exit code, `test result:` line for 3 and 4, the three actual
strings and OK/FAIL, then OVERALL: PASS or FAIL. Under 150 words.
