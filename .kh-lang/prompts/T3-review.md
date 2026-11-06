# T3 review (read-only, no commands)

Review these three files only:
- crates/kh-lang-core/src/dictionary.rs
- crates/kh-lang-core/src/matcher.rs
- crates/kh-lang-core/src/lib.rs

Product: Khmer word segmentation core. Invariants:
- Cluster-safe: tokens never split a Khmer cluster (coeng U+17D2 binds to the next base;
  combining marks U+17B6-U+17C8 attach to the preceding base).
- Lossless: concatenating returned byte ranges reproduces the input exactly, including
  OOV text; no character is ever dropped.
- bimm: forward + backward maximal matching, fewest tokens wins, deterministic tie.
- viterbi: DP over cluster boundaries; scores ln(freq/total); OOV spans split per cluster
  with a documented penalty; deterministic.
- dictionary parser: lenient, never panics; empty word column skipped; missing/non-numeric
  freq -> DEFAULT_FREQ; CRLF handled; embedded dict loads lazily via OnceLock.
- Core purity: zero external deps, no napi/wasm, no unwrap/expect outside tests.

Hunt specifically for: off-by-one in cluster byte ranges; ranges not covering the whole
string; backward path misalignment; freq total desync on insert/replace; panic paths;
O(n^2)-blowup on long runs that is trivially avoidable.

Output: numbered findings, each BLOCKING or NIT, `path:line`, one-line reason, concrete
fix. Then APPROVE or REQUEST-CHANGES. Under 200 words. Do not edit or run anything.
