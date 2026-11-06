import test from 'node:test'
import assert from 'node:assert/strict'
import { Segmenter, normalize, segment } from '../index.js'
import {
  BOM,
  ZWSP,
  assertTokenInvariants,
  countZwsp,
  isKhmerCodeUnit,
  stripBomAndZwsp,
  stripZwsp,
  tokenTexts,
} from './helpers.mjs'

const MODES = ['bimm', 'viterbi']

const validInputs = [
  ['empty', ''],
  ['one Khmer word', 'ខ្ញុំ'],
  ['Khmer compounds', 'ខ្ញុំស្រលាញ់ភ្នំពេញ'],
  ['Latin', 'Hello, world!'],
  ['ASCII digits', '0123456789'],
  ['Khmer digits', '០១២៣៤៥៦៧៨៩'],
  ['whitespace', ' \t\n\r\u00A0'],
  ['punctuation', '.,;:!?()[]{}«»—…'],
  ['emoji', '😀🇰🇭👨‍👩‍👧‍👦'],
  ['ZWSP', `ខ្ញុំ${ZWSP}ស្រលាញ់`],
  ['BOM', `${BOM}ខ្ញុំ`],
  ['mixed', `ខ្ញុំ loves ភ្នំពេញ 123 😀${ZWSP}!`],
  ['trailing coeng', 'ខ្ញុំក\u17D2'],
  ['lone coeng', '\u17D2'],
  ['high code points', '\u{10FFFF}\u{1D11E}'],
]

test('segments the documented example exactly', () => {
  assert.deepEqual(segment('ខ្ញុំស្រលាញ់ភ្នំពេញ'), ['ខ្ញុំ', 'ស្រលាញ់', 'ភ្នំពេញ'])
})

test('bimm is the default mode', () => {
  const text = 'ខ្ញុំស្រលាញ់ភ្នំពេញ'
  assert.deepEqual(segment(text), segment(text, { mode: 'bimm' }))
  assert.deepEqual(
    tokenTexts(new Segmenter().segment(text)),
    tokenTexts(new Segmenter({ mode: 'bimm' }).segment(text)),
  )
  assert.deepEqual(segment(text, { mode: 'viterbi' }).join(''), text)
})

test('segmentation is lossless for both modes', () => {
  for (const [label, text] of validInputs) {
    for (const mode of MODES) {
      assert.equal(segment(text, { mode }).join(''), text, `${label} (${mode})`)
      assert.equal(
        tokenTexts(new Segmenter({ mode }).segment(text)).join(''),
        text,
        `${label} (${mode}) class`,
      )
    }
  }
})

test('normalize only removes ZWSP and BOM from well-formed input', () => {
  for (const [label, text] of validInputs) {
    assert.equal(normalize(text), stripBomAndZwsp(text), label)
  }
  assert.equal(normalize('ក្សា'), 'ក្សា')
  assert.equal(normalize(`ក${ZWSP}្សា${BOM}`), 'ក្សា')
  assert.equal(normalize(`a${ZWSP}b${BOM}c`), 'abc')
})

test('tokens have exact UTF-16 offsets and are contiguous', () => {
  const inputs = [...validInputs.map(([, text]) => text), 'a😀b', '😀😀', 'ក\u17D2']

  for (const text of inputs) {
    for (const mode of MODES) {
      const segmenter = new Segmenter({ mode })
      const tokens = segmenter.segment(text)
      assertTokenInvariants(text, tokens)
      assert.deepEqual(tokenTexts(tokens), segment(text, { mode }))
      assert.deepEqual(segmenter.split(text), tokens)
      assert.deepEqual(tokenTexts(segmenter.split(text)), tokenTexts(tokens))
    }
  }
})

test('empty input stays empty', () => {
  for (const mode of MODES) {
    assert.deepEqual(segment('', { mode }), [])
    const segmenter = new Segmenter({ mode })
    assert.deepEqual(segmenter.segment(''), [])
    assert.deepEqual(segmenter.split(''), [])
    assert.equal(segmenter.insertZwsp(''), '')
  }
})

test('insertZwsp restores the input and is idempotent', () => {
  const segmenter = new Segmenter()
  const plain = [
    'ខ្ញុំស្រលាញ់ភ្នំពេញ',
    'សូមស្វាគមន៍មកកាន់ប្រទេសកម្ពុជា',
    'ខ្ញុំ',
    'abc ខ្ញុំ 123',
    'ក្សា',
  ]

  for (const text of plain) {
    const output = segmenter.insertZwsp(text)
    assert.equal(stripZwsp(output), text)
    assert.equal(segmenter.insertZwsp(output), output)
  }

  const multiWord = [
    'ខ្ញុំស្រលាញ់ភ្នំពេញ',
    'សូមស្វាគមន៍មកកាន់ប្រទេសកម្ពុជា',
    'ក្សាក្សា',
  ]
  for (const text of multiWord) {
    assert.ok(countZwsp(segmenter.insertZwsp(text)) > countZwsp(text))
  }
  assert.equal(
    segmenter.insertZwsp('ខ្ញុំស្រលាញ់ភ្នំពេញ'),
    `ខ្ញុំ${ZWSP}ស្រលាញ់${ZWSP}ភ្នំពេញ`,
  )
})

test('insertZwsp keeps pre-existing ZWSP', () => {
  const segmenter = new Segmenter()
  for (const text of [
    `ខ្ញុំ${ZWSP}ស្រលាញ់`,
    `ខ្ញុំស្រលាញ់${ZWSP}ភ្នំពេញ`,
    `a${ZWSP}b ${ZWSP} ខ្ញុំ`,
  ]) {
    const output = segmenter.insertZwsp(text)
    assert.equal(stripZwsp(output), stripZwsp(text))
    assert.ok(countZwsp(output) >= countZwsp(text))
    assert.equal(segmenter.insertZwsp(output), output)
  }
})

test('insertZwsp never touches Latin, ASCII digits, whitespace, or emoji', () => {
  const segmenter = new Segmenter()
  const boundaries = [
    'abcខ្ញុំ',
    'ខ្ញុំabc',
    '123ខ្ញុំ',
    'ខ្ញុំ123',
    ' ខ្ញុំ',
    'ខ្ញុំ ',
    'ខ្ញុំ\nស្រលាញ់',
    'ខ្ញុំ\tស្រលាញ់',
    'ខ្ញុំ😀ស្រលាញ់',
    'ខ្ញុំ!ស្រលាញ់',
    'ខ្ញុំ-ស្រលាញ់',
  ]
  for (const text of boundaries) {
    assert.equal(segmenter.insertZwsp(text), text, JSON.stringify(text))
  }
})

test('insertZwsp only inserts between Khmer letters and skips coeng tails', () => {
  const segmenter = new Segmenter()
  const output = segmenter.insertZwsp('ខ្ញុំស្រលាញ់ភ្នំពេញ')

  for (let index = 0; index < output.length; index += 1) {
    if (output[index] !== ZWSP) continue
    assert.ok(isKhmerCodeUnit(output.charCodeAt(index - 1)))
    assert.ok(isKhmerCodeUnit(output.charCodeAt(index + 1)))
  }

  // A word ending in coeng U+17D2 must not gain a boundary after the coeng.
  assert.ok(!/\u17D2\u200B/u.test(output))
  assert.ok(!/\u17D2\u200B/u.test(segmenter.insertZwsp('ខ្ញុំក\u17D2ខ្ញុំ')))
  assert.equal(segmenter.insertZwsp('១២៣'), '១២៣')
})
