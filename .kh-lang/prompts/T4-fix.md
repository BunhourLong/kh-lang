# T4 — reviewer NIT fixes (builder)

Repo: /Users/capon/Documents/Projects/kh-lang
Read: `.kh-lang/prompts/T4.md`, `.kh-lang/prompts/T4-review.md`, and the current
`crates/kh-lang-core/src/segmenter.rs` + `lib.rs`.

Allowed files (nothing else):
- `crates/kh-lang-core/src/segmenter.rs`
- `crates/kh-lang-core/src/lib.rs`

Fix these reviewer NITs (numbers match T4-review.md):

1. Make the dictionary-override test discriminating: override with a table containing
   a compound the embedded dict does not have as one word (e.g. `ខ្ញុំភ្នំពេញ`), assert
   it becomes one token.
2. Test extras layered on an override: `.dictionary(Dictionary::new()).add_word(x)` →
   `x` is one token.
3. `insert_zwsp` doc: say the **inserted** U+200B are removed by the round-trip, not
   pre-existing ones.
4. Qualify `Token`/`segment` docs: u32 offsets saturate only beyond u32::MAX UTF-16
   units (unreachable from JS); below that, contiguity/length guarantees hold.
5. `insert_zwsp`: never insert a ZWSP when the left token ends with coeng U+17D2
   (would split the coeng-base pair).
6. `insert_zwsp`: never insert a ZWSP between Khmer digits U+17E0..U+17E9 (would split a
   number).
7. `lib.rs` module docs: add `segmenter` to the module list; remove/adjust the stale
   "Later tasks layer segmentation" sentence.

Add one test each for 5 and 6. Keep all existing tests green. No new deps, no commits,
no scope creep beyond the seven items.

DONE: from `crates/kh-lang-core`, exit 0:
`cargo fmt --all --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`,
`cargo test --doc`.

Report only: files changed, last 3 lines per command, blockers.
