import test from 'node:test'
import assert from 'node:assert/strict'
import { Segmenter, normalize, segment } from '../index.js'
import {
  BOM,
  ZWSP,
  assertTokenInvariants,
  stripBomAndZwsp,
  stripZwsp,
  tokenTexts,
  wellFormed,
} from './helpers.mjs'

const MODES = ['bimm', 'viterbi']

/** Deterministic 32-bit PRNG (mulberry32). */
function mulberry32(seed) {
  let state = seed >>> 0
  return function next() {
    state = (state + 0x6d2b79f5) >>> 0
    let value = state
    value = Math.imul(value ^ (value >>> 15), value | 1)
    value ^= value + Math.imul(value ^ (value >>> 7), value | 61)
    return ((value ^ (value >>> 14)) >>> 0) / 0x1_0000_0000
  }
}

const khmerConsonants = [...'កខគឃងចឆជឈញដឋឌឍណតថទធនបផពភមយរលវសហឡអ']

const alphabet = [
  ...khmerConsonants,
  ...'ាិីឹឺុូួើឿៀេែៃោៅ',
  ...'ំះៈ',
  '\u17D2',
  ...'០១២៣៤៥៦៧៨៩',
  ...'abcXYZ',
  ...'0123456789',
  ' ',
  '\t',
  '\n',
  '\r',
  '.',
  ',',
  '!',
  '?',
  '😀',
  '🇰🇭',
  '\u{10FFFF}',
  ZWSP,
  BOM,
]

/**
 * Builds a deterministic, well-formed UTF-16 corpus. Coeng is always followed
 * by a base character so the generated text is still valid Khmer.
 */
function corpus() {
  const random = mulberry32(0x5eed_1234)
  const strings = ['']

  for (let count = 0; count < 300; count += 1) {
    const length = Math.floor(random() * 30)
    let text = ''
    for (let index = 0; index < length; index += 1) {
      let unit = alphabet[Math.floor(random() * alphabet.length)]
      if (unit === '\u17D2') unit += khmerConsonants[Math.floor(random() * khmerConsonants.length)]
      text += unit
    }
    strings.push(text)
  }

  return strings
}

function assertLossless(text) {
  for (const mode of MODES) {
    const values = segment(text, { mode })
    assert.equal(values.join(''), text, `${mode} top-level losslessness`)

    const segmenter = new Segmenter({ mode })
    const tokens = segmenter.segment(text)
    assertTokenInvariants(text, tokens)
    assert.equal(tokenTexts(tokens).join(''), text)
    assert.deepEqual(segmenter.split(text), tokens)

    assert.equal(normalize(text), stripBomAndZwsp(text))
  }
}

test('well-formed arbitrary input never panics and stays lossless', () => {
  for (const text of corpus()) assertLossless(text)
})

test('malformed UTF-16 is sanitized at the N-API boundary, never fatal', () => {
  // napi-rs converts JS strings to Rust `String` through UTF-8, where lone
  // surrogates have no representation, so the native binding replaces them
  // with U+FFFD (same as the `wellFormed` helper). This pins that boundary
  // behavior: no call throws and the process keeps running.
  const malformed = [
    '\uD800',
    '\uDC00',
    'a\uD800b',
    '\uD800\uD800',
    '\uDC00\uD800',
    'ខ្ញុំ\uD800ស្រលាញ់',
    'ខ្ញុំ\uDC00',
    '\uD83D',
    '\uDE00',
    '\uD83D\uD83D\uDE00',
  ]

  for (const text of malformed) {
    const sanitized = wellFormed(text)
    for (const mode of MODES) {
      const values = segment(text, { mode })
      assert.equal(values.join(''), sanitized, `${mode} sanitized`)

      const segmenter = new Segmenter({ mode })
      const tokens = segmenter.segment(text)
      assertTokenInvariants(sanitized, tokens)
      assert.equal(tokenTexts(tokens).join(''), sanitized)
      assert.deepEqual(segmenter.split(text), tokens)

      assert.equal(normalize(text), sanitized.replaceAll(ZWSP, '').replaceAll(BOM, ''))
      assert.equal(stripZwsp(segmenter.insertZwsp(text)), stripZwsp(sanitized))
    }
  }

  // The process is still healthy after every malformed call.
  assert.deepEqual(segment('ខ្ញុំស្រលាញ់ភ្នំពេញ'), ['ខ្ញុំ', 'ស្រលាញ់', 'ភ្នំពេញ'])
})
