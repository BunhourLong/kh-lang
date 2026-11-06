# T4 review (read-only, no commands)

Review ONLY these changed code files against `.kh-lang/tasks.md` T4:
- `crates/kh-lang-core/src/segmenter.rs` (new)
- `crates/kh-lang-core/src/lib.rs` (module decl + re-exports)

Product: Khmer word segmentation core. Required invariants:
- `segment` produces `Token { text, start: u32, end: u32 }` with UTF-16 code-unit
  offsets; `end - start == text.encode_utf16().count()`; offsets contiguous from 0 to
  the input's UTF-16 length; empty input → empty vec.
- Lossless for ANY input: concatenating token texts reproduces the input exactly,
  including ZWSP U+200B, BOM U+FEFF, whitespace, Latin, digits, emoji. `segment` must
  NOT call `normalize` (that would drop invisible chars).
- `RunKind::Other` runs → exactly one token. `RunKind::Khmer` runs → bimm or viterbi
  per `Mode`; no cluster may be split (matcher guarantees cluster boundaries).
- `Segmenter::new()` uses the embedded dict and does not deep-clone it (Cow or
  equivalent). Builder override/extras semantics are documented and tested.
- `insert_zwsp` inserts U+200B only between adjacent Khmer-Khmer token boundaries;
  never at edges, never next to existing ZWSP/whitespace/Latin; idempotent; removing
  inserted ZWSP restores the original.
- No unwrap/expect outside tests; no panic on arbitrary input; u32 conversion cannot
  panic. Zero external deps, no napi/wasm types. Public items documented.
- Tests genuinely exercise the above (not vacuous), deterministic, no ignored tests.

Hunt specifically for: UTF-16 vs byte offset mixups; byte-slicing panics on non-ASCII;
sample/char boundary bugs; `insert_zwsp` duplicating existing ZWSP or inserting at
non-Khmer boundaries; builder extras lost when a dictionary override is present;
`Cow` clone accidentally done in `new()`; over-broad API or scope creep.

Output: numbered findings, each BLOCKING or NIT, `path:line`, one-line reason, concrete
fix. Then APPROVE or REQUEST-CHANGES. Under 200 words. Do not edit or run anything.
