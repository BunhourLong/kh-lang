# T5 — final verification (tester, no edits)

Repo: /Users/capon/Documents/Projects/kh-lang
Read `.kh-lang/prompts/T5.md` for the intended surface. Do NOT edit any file and do NOT
commit. Run the checks below from the repo root with
`export PATH="$HOME/.cargo/bin:$PATH"`.

## Build + deterministic checks

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
npm ci
npx napi build --platform --no-const-enum
npx tsc --noEmit
git diff --check
```

## Acceptance smoke (node, heredoc; assert and exit non-zero on failure)

1. `segment("ខ្ញុំស្រលាញ់ភ្នំពេញ")` equals `["ខ្ញុំ","ស្រលាញ់","ភ្នំពេញ"]`.
2. Losslessness: `segment(s).join("") === s` for
   `"ខ្ញុំ Hello, 123! 🙂 ភ្នំពេញ"` and for
   `"ខ្ញុំ\u200Bស្រលាញ់\uFEFFភ្នំពេញ"` (ZWSP/BOM preserved).
3. Default mode equals `{ mode: "bimm" }`; `{ mode: "viterbi" }` returns a lossless
   segmentation of the same input.
4. `normalize("ក\u200Bខ") === "កខ"` and preserves Khmer marks otherwise.
5. Class: `new Segmenter().split("ខ្ញុំ")` returns a `Token` with exact `text`, numeric
   `start`/`end`, and `end - start === text.length` for BMP Khmer; `segment` and
   `split` agree.
6. `insertZwsp("ខ្ញុំស្រលាញ់").replace(/\u200B/g,"")` equals the input and is idempotent.
7. Custom words additive:
   `new Segmenter().addWords(["ខ្ញុំស្រលាញ់"]).segment("ខ្ញុំស្រលាញ់")` is one token,
   and the embedded dictionary still segments `ភ្នំពេញ`.
   Constructor `{ dictionary: ["ខ្ញុំស្រលាញ់"] }` behaves additively too; object form
   `{ "ខ្ញុំស្រលាញ់": 500 }` works; `addWords` chaining returns a Segmenter.
8. Errors: bad mode → `.code === "ERR_KH_OPTIONS"`; `addWords([""])`,
   `addWords(["a\tb"])`, `addWords({"a": 0})`, `addWords({"a": NaN})` →
   `.code === "ERR_KH_WORDS"`; no process abort.
9. `segment("\uD800")` either returns or throws a normal `Error`; process survives.
10. `index.d.ts` contains the exported `Token`, `SegmentOptions`, `SegmenterOptions`,
    `SegmentMode = 'bimm' | 'viterbi'`, and `addWords(...): Segmenter`.
11. `git status --short` shows only T5 files/artifacts (plus `.kh-lang` state), no
    changes under `crates/kh-lang-core/`, `data/`, or `scripts/`.

Report a compact table: check, PASS/FAIL, and for failures the exact command, exit
code, and last 5 lines. End with `ALL PASS` or `FAILURES: <n>`. Do not paste full logs.
