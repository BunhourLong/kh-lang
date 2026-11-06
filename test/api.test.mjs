import test from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import { Segmenter, normalize, segment } from '../index.js'
import { tokenTexts } from './helpers.mjs'

const here = dirname(fileURLToPath(import.meta.url))
const root = join(here, '..')

const CUSTOM_WORD = 'ស្រលាញ់ភ្នំពេញ'
const OTHER_CUSTOM_WORD = 'ភ្នំពេញខ្ញុំ'

/** @param {() => unknown} fn @param {string} code */
function assertThrowsCode(fn, code) {
  assert.throws(fn, (error) => {
    assert.ok(error instanceof Error)
    assert.equal(error.code, code)
    return true
  })
}

test('package exposes the documented named exports', () => {
  assert.equal(typeof segment, 'function')
  assert.equal(typeof normalize, 'function')
  assert.equal(typeof Segmenter, 'function')

  const instance = new Segmenter()
  for (const method of ['segment', 'split', 'insertZwsp', 'addWords']) {
    assert.equal(typeof instance[method], 'function', method)
  }
})

test('index.d.ts documents the public surface and is listed in package.json', () => {
  const types = readFileSync(join(root, 'index.d.ts'), 'utf8')
  for (const fragment of [
    'export declare function segment',
    'export declare function normalize',
    'export declare class Segmenter',
    'export type SegmentMode',
    'export interface SegmenterOptions',
    'export interface SegmentOptions',
    'export interface Token',
    'insertZwsp',
    'addWords',
    'dictionary',
    'start: number',
    'end: number',
  ]) {
    assert.ok(types.includes(fragment), fragment)
  }

  const pkg = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8'))
  assert.equal(pkg.name, 'kh-lang')
  assert.equal(pkg.types, 'index.d.ts')
  assert.ok(pkg.files.includes('index.js'))
  assert.ok(pkg.files.includes('index.d.ts'))
  assert.ok(pkg.files.includes('licenses'))
})

test('constructor dictionary arrays add words without replacing embedded ones', () => {
  const segmenter = new Segmenter({ dictionary: [CUSTOM_WORD] })

  assert.deepEqual(tokenTexts(segmenter.segment(CUSTOM_WORD)), [CUSTOM_WORD])
  // Embedded words still segment: custom words are additive.
  assert.deepEqual(tokenTexts(segmenter.segment('ខ្ញុំ')), ['ខ្ញុំ'])
  assert.deepEqual(
    segment('ខ្ញុំស្រលាញ់ភ្នំពេញ'),
    ['ខ្ញុំ', 'ស្រលាញ់', 'ភ្នំពេញ'],
  )
})

test('constructor dictionary objects accept explicit frequencies', () => {
  const segmenter = new Segmenter({
    mode: 'bimm',
    dictionary: { [CUSTOM_WORD]: 50_000 },
  })

  assert.deepEqual(tokenTexts(segmenter.segment(CUSTOM_WORD)), [CUSTOM_WORD])
  assert.deepEqual(tokenTexts(segmenter.segment('ខ្ញុំ')), ['ខ្ញុំ'])
})

test('addWords is additive, chains, and returns a Segmenter', () => {
  const segmenter = new Segmenter()
  const returned = segmenter.addWords([CUSTOM_WORD])

  assert.ok(returned instanceof Segmenter)
  assert.deepEqual(tokenTexts(returned.segment(CUSTOM_WORD)), [CUSTOM_WORD])
  assert.deepEqual(tokenTexts(segmenter.segment(CUSTOM_WORD)), [CUSTOM_WORD])

  const chained = new Segmenter()
    .addWords([CUSTOM_WORD])
    .addWords({ [OTHER_CUSTOM_WORD]: 25_000 })
  assert.ok(chained instanceof Segmenter)
  assert.deepEqual(tokenTexts(chained.segment(CUSTOM_WORD)), [CUSTOM_WORD])
  assert.deepEqual(tokenTexts(chained.segment(OTHER_CUSTOM_WORD)), [OTHER_CUSTOM_WORD])
  assert.deepEqual(tokenTexts(chained.segment('ខ្ញុំ')), ['ខ្ញុំ'])
})

test('invalid modes and options throw ERR_KH_OPTIONS', () => {
  for (const mode of ['nope', '', 'Bimm', 42]) {
    assertThrowsCode(() => segment('ខ្ញុំ', { mode }), 'ERR_KH_OPTIONS')
    assertThrowsCode(() => new Segmenter({ mode }), 'ERR_KH_OPTIONS')
  }
})

test('malformed option shapes throw ERR_KH_OPTIONS', () => {
  const malformed = [
    () => segment('x', 42),
    () => segment('x', { mode: 5 }),
    () => new Segmenter('x'),
    () => new Segmenter({ dictionary: 42 }),
  ]

  for (const call of malformed) {
    assertThrowsCode(call, 'ERR_KH_OPTIONS')
  }

  // The process survived every malformed call.
  assert.deepEqual(segment('ខ្ញុំ'), ['ខ្ញុំ'])
})

test('invalid custom words throw ERR_KH_WORDS', () => {
  for (const word of ['', '   ', 'a\tb', 'a\rb', 'a\nb']) {
    assertThrowsCode(() => new Segmenter({ dictionary: [word] }), 'ERR_KH_WORDS')
    assertThrowsCode(() => new Segmenter().addWords([word]), 'ERR_KH_WORDS')
  }
})

test('invalid frequencies throw ERR_KH_WORDS', () => {
  for (const frequency of [0, -1, Number.NaN, Number.POSITIVE_INFINITY, 2 ** 32]) {
    assertThrowsCode(
      () => new Segmenter({ dictionary: { [CUSTOM_WORD]: frequency } }),
      'ERR_KH_WORDS',
    )
    assertThrowsCode(
      () => new Segmenter().addWords({ [CUSTOM_WORD]: frequency }),
      'ERR_KH_WORDS',
    )
  }
})

test('null and omitted options use the defaults', () => {
  assert.deepEqual(segment('ខ្ញុំ', null), ['ខ្ញុំ'])
  assert.deepEqual(segment('ខ្ញុំ', undefined), ['ខ្ញុំ'])
  assert.deepEqual(tokenTexts(new Segmenter(null).segment('ខ្ញុំ')), ['ខ្ញុំ'])
})
