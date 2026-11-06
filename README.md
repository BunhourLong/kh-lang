# kh-lang

Khmer word segmentation for Node.js, powered by a pure-Rust core.

`kh-lang` splits Khmer text into words without losing characters: for
well-formed JavaScript strings, concatenating the returned tokens reproduces the
input exactly. It ships with the SIL Khmer Line Breaking Dictionary and exposes
one function, a reusable `Segmenter` class, and a small normalizer.

## Install

```sh
npm install kh-lang
```

Requires Node.js 18 or newer. The package is a native addon built with
[napi-rs](https://napi.rs/). Prebuilt binaries ship for macOS (x64/arm64),
Linux glibc (x64/arm64), and Windows (x64) as platform packages
(`kh-lang-<platform>`) that npm installs automatically through
`optionalDependencies`. Every other platform needs a local Rust toolchain; see
[Build from source](#build-from-source).

## Quick start

```js
import { segment } from 'kh-lang'

console.log(segment('ខ្ញុំស្រលាញ់ភ្នំពេញ'))
// [ 'ខ្ញុំ', 'ស្រលាញ់', 'ភ្នំពេញ' ]
```

CommonJS works too:

```js
const { segment, Segmenter, normalize } = require('kh-lang')
```

## API

### `segment(text, options?)`

Returns `string[]`: the token texts, in order. `options.mode` is `'bimm'`
(default) or `'viterbi'`.

```js
segment('ខ្ញុំស្រលាញ់ភ្នំពេញ', { mode: 'viterbi' })
```

### `new Segmenter(options?)`

Creates a reusable segmenter.

```js
const segmenter = new Segmenter({
  mode: 'bimm',
  dictionary: ['ស្រលាញ់ភ្នំពេញ'],
})
```

- `mode` — `'bimm'` (default) or `'viterbi'`.
- `dictionary` — custom words as `string[]`, or as `Record<string, number>`
  mapping each word to a frequency. Custom words are **additive**: they layer on
  top of the embedded dictionary and never replace it.

Methods:

- `.segment(text)` — returns `Token[]`.
- `.split(text)` — alias for `.segment(text)`.
- `.insertZwsp(text)` — returns text with U+200B ZWSP between adjacent Khmer
  words. Existing ZWSP are preserved; removing the inserted ZWSP restores the
  input.
- `.addWords(words)` — adds words (array or frequency map), mutates the
  receiver, and returns a `Segmenter` so calls can chain.

A `Token` has three fields:

```ts
interface Token {
  text: string  // exact characters from the input
  start: number // UTF-16 code-unit offset of the first code unit
  end: number   // UTF-16 code-unit offset one past the last code unit
}
```

Offsets are UTF-16 units, matching `String.prototype.slice` and
`String.prototype.length`. Tokens are contiguous, so
`text.slice(tokens[0].start, tokens.at(-1).end) === text`.

### Losslessness and malformed input

For well-formed JavaScript strings, `segment`, `Segmenter#segment`,
`Segmenter#split`, and `insertZwsp` are lossless. Lone or unpaired UTF-16
surrogates are not representable in UTF-8, so the napi-rs string conversion
replaces them with U+FFFD before the Rust core sees them; those code units come
back as U+FFFD instead of the original value. The library never panics or
aborts the process on arbitrary input, including invalid or truncated UTF-16.

### `normalize(text)`

Removes U+200B ZWSP and U+FEFF BOM only. Every other character, including Khmer
combining marks, is preserved unchanged and never reordered.

```js
normalize('ខ្ញុំ\u200Bស្រលាញ់') // 'ខ្ញុំស្រលាញ់'
```

### Errors

Inputs are validated in Rust. Invalid values throw an ordinary `Error` with a
stable code:

- `ERR_KH_OPTIONS` — unknown mode or malformed options.
- `ERR_KH_WORDS` — empty/whitespace-only word, a word containing tab, CR, or
  LF, or a non-finite, non-positive, or out-of-range frequency.

## Modes

- **`bimm`** (default, deterministic) — bidirectional maximal matching against
  the embedded dictionary, falling back to one token per grapheme cluster for
  unknown text.
- **`viterbi`** — dynamic programming over dictionary frequencies; it can prefer
  a more probable split when several valid segmentations exist.

For well-formed strings both modes are lossless and treat Latin, digits,
whitespace, punctuation, and other non-Khmer runs as single tokens.

## Custom words

Custom words are additive in both the constructor and `.addWords()`:

```js
import { Segmenter } from 'kh-lang'

const segmenter = new Segmenter()
  .addWords(['ស្រលាញ់ភ្នំពេញ'])
  .addWords({ 'ភ្នំពេញខ្ញុំ': 25000 })

segmenter.segment('ស្រលាញ់ភ្នំពេញ').map((token) => token.text)
// [ 'ស្រលាញ់ភ្នំពេញ' ]
```

The embedded dictionary still works after adding custom words.

## Build from source

Prerequisites: Node.js 18+, npm, and a stable Rust toolchain with `rustfmt` and
`clippy`.

```sh
git clone https://github.com/BunhourLong/kh-lang.git
cd kh-lang
npm ci
npm run build   # release addon
npm test        # builds a debug addon and runs the node:test suite
```

For development, `npm run build:debug` builds without optimizations. The core
crate lives in `crates/kh-lang-core` and has no external dependencies; the
napi-rs glue is in `src/index.rs`.

## ភាសាខ្មែរ

`kh-lang` គឺជាបណ្ណាល័យបំបែកពាក្យខ្មែរសម្រាប់ Node.js ដែលសរសេរដោយ Rust។
វាប្រើវចនានុក្រមខ្មែរដែលបានបញ្ចូលមកជាមួយ ដើម្បីបំបែកប្រយោគជាពាក្យ
ដោយមិនបាត់តួអក្សរណាមួយឡើយ សម្រាប់ខ្សែអក្សរ UTF-16 ដែលត្រឹមត្រូវ។

```js
import { segment } from 'kh-lang'

const text = 'ខ្ញុំស្រលាញ់ភ្នំពេញ'
console.log(segment(text))
// [ 'ខ្ញុំ', 'ស្រលាញ់', 'ភ្នំពេញ' ]
```

លទ្ធផលដែលបានត្រឡប់មកវិញ គឺជាបញ្ជីពាក្យដែលនៅជាប់គ្នា។ ប្រសិនបើអ្នក
បញ្ចូលពាក្យទាំងអស់គ្នាវិញ អ្នកនឹងទទួលបានអត្ថបទដើមដូចដើម។ អ្នកក៏អាច
ប្រើ `Segmenter` ដើម្បីបន្ថែមពាក្យថ្មីៗ និងជ្រើសរើសរបៀបបំបែក `viterbi`
បានផងដែរ។

## License

MIT © 2026 BunhourLong. See [LICENSE](LICENSE).

The embedded dictionary comes from the SIL Khmer Line Breaking Dictionary
(`khmerlbdict`) and is used under the MIT license; see
[`licenses/khmerlbdict-LICENSE`](licenses/khmerlbdict-LICENSE).
