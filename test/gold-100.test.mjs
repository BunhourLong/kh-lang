import test from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import { Segmenter } from '../index.js'
import { gold100 } from './fixtures/gold-100.mjs'
import { assertTokenInvariants, tokenTexts } from './helpers.mjs'

const here = dirname(fileURLToPath(import.meta.url))
const root = join(here, '..')

const dictionary = new Set(
  readFileSync(join(root, 'data', 'khmer.lbdict.txt'), 'utf8')
    .split('\n')
    .map((line) => line.split('\t')[0])
    .filter(Boolean),
)

/** @param {string[]} tokens */
function boundaries(tokens) {
  const result = new Set()
  let offset = 0
  for (let index = 0; index < tokens.length - 1; index += 1) {
    offset += tokens[index].length
    result.add(offset)
  }
  return result
}

/** @param {Set<number>} offsets */
function signature(offsets) {
  return [...offsets].sort((a, b) => a - b).join(',')
}

/**
 * Micro-averaged boundary F1 over the whole fixture.
 *
 * @param {{ text: string, gold: string[] }[]} sentences
 * @param {Segmenter} segmenter
 */
function boundaryF1(sentences, segmenter) {
  let truePositive = 0
  let falsePositive = 0
  let falseNegative = 0

  for (const { text, gold } of sentences) {
    const tokens = segmenter.segment(text)
    assertTokenInvariants(text, tokens)
    assert.equal(tokenTexts(tokens).join(''), text)

    const predicted = boundaries(tokenTexts(tokens))
    const expected = boundaries(gold)
    for (const offset of predicted) {
      if (expected.has(offset)) truePositive += 1
      else falsePositive += 1
    }
    for (const offset of expected) {
      if (!predicted.has(offset)) falseNegative += 1
    }
  }

  const precision = truePositive / (truePositive + falsePositive)
  const recall = truePositive / (truePositive + falseNegative)
  return (2 * precision * recall) / (precision + recall)
}

test('gold fixture is 100 dictionary-built sentences', () => {
  assert.equal(gold100.length, 100)
  const seen = new Set()

  for (const { text, gold } of gold100) {
    assert.equal(gold.join(''), text)
    assert.ok(gold.length >= 2 && gold.length <= 6, text)
    assert.equal(new Set(gold).size, gold.length, 'gold words must be distinct')
    assert.ok(!seen.has(text), `duplicate sentence: ${text}`)
    seen.add(text)
    assert.match(text, /^[\u1780-\u17FF]+$/u, 'pure Khmer sentence')
    for (const word of gold) {
      assert.ok(dictionary.has(word), `not an embedded headword: ${word}`)
    }
  }
})

test('fixture includes at least 25 mode-disagreement sentences', () => {
  const bimm = new Segmenter({ mode: 'bimm' })
  const viterbi = new Segmenter({ mode: 'viterbi' })
  let disagreements = 0

  for (const { text } of gold100) {
    const bimmBoundaries = boundaries(tokenTexts(bimm.segment(text)))
    const viterbiBoundaries = boundaries(tokenTexts(viterbi.segment(text)))
    if (signature(bimmBoundaries) !== signature(viterbiBoundaries)) {
      disagreements += 1
    }
  }

  assert.ok(
    disagreements >= 25,
    `expected at least 25 mode disagreements, got ${disagreements}`,
  )
})

test('viterbi boundary F1 is at least bimm boundary F1', () => {
  const bimmF1 = boundaryF1(gold100, new Segmenter({ mode: 'bimm' }))
  const viterbiF1 = boundaryF1(gold100, new Segmenter({ mode: 'viterbi' }))

  assert.ok(Number.isFinite(bimmF1))
  assert.ok(Number.isFinite(viterbiF1))
  assert.ok(
    viterbiF1 >= bimmF1,
    `viterbi F1 ${viterbiF1.toFixed(4)} < bimm F1 ${bimmF1.toFixed(4)}`,
  )
})
