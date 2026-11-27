import test from 'node:test'
import assert from 'node:assert/strict'
import { readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { dirname, join } from 'node:path'
import { runInNewContext } from 'node:vm'
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

test('addWords returns the exact receiver', () => {
  const segmenter = new Segmenter()
  const returned = segmenter.addWords([CUSTOM_WORD])
  assert.strictEqual(returned, segmenter)

  returned.addWords([OTHER_CUSTOM_WORD])
  assert.deepEqual(tokenTexts(segmenter.segment(OTHER_CUSTOM_WORD)), [OTHER_CUSTOM_WORD])
  segmenter.addWords({ abc: 10 })
  assert.deepEqual(tokenTexts(returned.segment(CUSTOM_WORD)), [CUSTOM_WORD])
})

test('failed addWords leaves the receiver unchanged', () => {
  const segmenter = new Segmenter({ dictionary: [CUSTOM_WORD] })
  const before = tokenTexts(segmenter.segment(OTHER_CUSTOM_WORD))
  assert.notDeepEqual(before, [OTHER_CUSTOM_WORD])

  assertThrowsCode(() => segmenter.addWords([OTHER_CUSTOM_WORD, '']), 'ERR_KH_WORDS')
  assertThrowsCode(() => segmenter.addWords(new Map([[OTHER_CUSTOM_WORD, 5]])), 'ERR_KH_WORDS')

  assert.deepEqual(tokenTexts(segmenter.segment(OTHER_CUSTOM_WORD)), before)
  assert.deepEqual(tokenTexts(segmenter.segment(CUSTOM_WORD)), [CUSTOM_WORD])
})

test('non-plain objects are rejected in every position', () => {
  const nonPlain = [
    () => new Date(),
    () => new Map(),
    () => new Set(),
    () => /regex/,
    () => Buffer.from('x'),
    () => new Uint8Array([1, 2]),
    () => new ArrayBuffer(4),
    () => new DataView(new ArrayBuffer(4)),
    () => new Error('x'),
    () => Promise.resolve(),
    () => new Segmenter(),
    () => Object.create({}),
    () => new (class Custom {})(),
    () => Object(1),
    () => Object('x'),
    () => Object.setPrototypeOf(new Date(), null),
    () => Object.setPrototypeOf(Buffer.from('x'), Object.prototype),
    () => runInNewContext('new Date()'),
    () => runInNewContext('new Map()'),
    () => Object.setPrototypeOf(new Map(), null),
    () => Object.setPrototypeOf(new Set(), null),
    () => Object.setPrototypeOf(new WeakMap(), null),
    () => Object.setPrototypeOf(new WeakSet(), null),
    () => Object.setPrototypeOf(/x/, null),
    () => Object.setPrototypeOf(Object(1), null),
    () => Object.setPrototypeOf(Object('x'), null),
    () => Object.setPrototypeOf(Object(true), null),
    () => Object.setPrototypeOf(Object(Symbol('s')), null),
    () => Object.setPrototypeOf(Object(1n), null),
    () => Object.setPrototypeOf(new SharedArrayBuffer(4), null),
    () => Object.create(Object.create(null)),
    () => {
      const fake = Object.create(null)
      fake.constructor = Object
      return Object.create(fake)
    },
    () => new Proxy(new Date(), {}),
    () => new Proxy({}, {}),
    () => new Proxy(Object.create(null), {}),
    () => new Proxy([], {}),
    () => runInNewContext('Object.setPrototypeOf(new Map(), null)'),
    () => runInNewContext('new Proxy({}, {})'),
    () => Object.setPrototypeOf(new WeakRef({}), null),
    () => Object.setPrototypeOf(new WeakRef({}), Object.prototype),
    () => Object.setPrototypeOf(new FinalizationRegistry(() => {}), null),
    () => Object.setPrototypeOf(new Segmenter(), null),
    () => Object.setPrototypeOf(new Segmenter(), Object.prototype),
    () => runInNewContext('Object.setPrototypeOf(new WeakRef({}), null)'),
  ]

  for (const make of nonPlain) {
    const label = String(make)
    assert.throws(() => segment('x', make()), { code: 'ERR_KH_OPTIONS' }, label)
    assert.throws(() => new Segmenter(make()), { code: 'ERR_KH_OPTIONS' }, label)
    assert.throws(() => new Segmenter({ dictionary: make() }), { code: 'ERR_KH_OPTIONS' }, label)
    assert.throws(() => new Segmenter().addWords(make()), { code: 'ERR_KH_WORDS' }, label)
  }

  for (const options of [[], ['a'], Object.setPrototypeOf([], null)]) {
    assertThrowsCode(() => segment('x', options), 'ERR_KH_OPTIONS')
    assertThrowsCode(() => new Segmenter(options), 'ERR_KH_OPTIONS')
  }

  for (const words of [null, undefined, 1, 'x', true, Symbol('x'), 10n]) {
    assertThrowsCode(() => new Segmenter().addWords(words), 'ERR_KH_WORDS')
  }
})

test('plain objects are accepted from any realm or with a null prototype', () => {
  const nullProto = Object.assign(Object.create(null), { mode: 'viterbi' })
  const foreign = runInNewContext("({ mode: 'viterbi' })")
  for (const options of [{}, { mode: 'viterbi' }, nullProto, foreign]) {
    assert.deepEqual(segment('ខ្ញុំ', options), ['ខ្ញុំ'])
    assert.deepEqual(tokenTexts(new Segmenter(options).segment('ខ្ញុំ')), ['ខ្ញុំ'])
  }

  const nullWords = Object.assign(Object.create(null), { [CUSTOM_WORD]: 50_000 })
  const foreignWords = runInNewContext('({ [word]: 50_000 })', { word: CUSTOM_WORD })
  for (const words of [{ [CUSTOM_WORD]: 50_000 }, nullWords, foreignWords]) {
    const fromOptions = new Segmenter({ dictionary: words })
    assert.deepEqual(tokenTexts(fromOptions.segment(CUSTOM_WORD)), [CUSTOM_WORD])
    const fromAdd = new Segmenter().addWords(words)
    assert.deepEqual(tokenTexts(fromAdd.segment(CUSTOM_WORD)), [CUSTOM_WORD])
  }
  assert.ok(new Segmenter({ dictionary: {} }) instanceof Segmenter)
  assert.ok(new Segmenter().addWords(Object.create(null)) instanceof Segmenter)
})

test('prototype pollution does not inject options or words', () => {
  const plain = new Segmenter()
  const baseline = tokenTexts(plain.segment(CUSTOM_WORD))
  assert.notDeepEqual(baseline, [CUSTOM_WORD])

  try {
    Object.prototype.mode = 'nope'
    Object.prototype.dictionary = 42
    Object.prototype[CUSTOM_WORD] = 50_000

    assert.deepEqual(segment('ខ្ញុំ', {}), ['ខ្ញុំ'])
    const fromOptions = new Segmenter({})
    assert.deepEqual(tokenTexts(fromOptions.segment(CUSTOM_WORD)), baseline)
    const fromDictionary = new Segmenter({ dictionary: { [OTHER_CUSTOM_WORD]: 25_000 } })
    assert.deepEqual(tokenTexts(fromDictionary.segment(CUSTOM_WORD)), baseline)
    const fromAdd = new Segmenter().addWords({ [OTHER_CUSTOM_WORD]: 25_000 })
    assert.deepEqual(tokenTexts(fromAdd.segment(CUSTOM_WORD)), baseline)
    assert.deepEqual(tokenTexts(fromAdd.segment(OTHER_CUSTOM_WORD)), [OTHER_CUSTOM_WORD])
  } finally {
    delete Object.prototype.mode
    delete Object.prototype.dictionary
    delete Object.prototype[CUSTOM_WORD]
  }
})

test('hostile Proxies are rejected before any trap runs', () => {
  let trapped = false
  const boom = () => {
    trapped = true
    throw new Error('boom')
  }
  const hostile = () =>
    new Proxy({ [CUSTOM_WORD]: 5 }, {
      getPrototypeOf: boom,
      getOwnPropertyDescriptor: boom,
      ownKeys: boom,
      get: boom,
      has: boom,
    })

  assertThrowsCode(() => segment('x', hostile()), 'ERR_KH_OPTIONS')
  assertThrowsCode(() => new Segmenter(hostile()), 'ERR_KH_OPTIONS')
  assertThrowsCode(() => new Segmenter({ dictionary: hostile() }), 'ERR_KH_OPTIONS')
  assertThrowsCode(() => new Segmenter().addWords(hostile()), 'ERR_KH_WORDS')
  assert.equal(trapped, false)
})

test('throwing own getters surface as an Error without aborting', () => {
  const throwing = (key) =>
    Object.defineProperty({}, key, {
      enumerable: true,
      get() {
        throw new Error('boom')
      },
    })

  assert.throws(() => segment('x', throwing('mode')), Error)
  assert.throws(() => new Segmenter(throwing('mode')), Error)
  assert.throws(() => new Segmenter(throwing('dictionary')), Error)
  assert.throws(() => new Segmenter({ dictionary: throwing(CUSTOM_WORD) }), Error)
  assert.throws(() => new Segmenter().addWords(throwing(CUSTOM_WORD)), Error)
  assert.deepEqual(segment('ខ្ញុំ'), ['ខ្ញុំ'])
})

test('sparse word arrays never read inherited entries', () => {
  const baseline = tokenTexts(new Segmenter().segment(CUSTOM_WORD))
  try {
    Array.prototype[0] = CUSTOM_WORD
    assertThrowsCode(() => new Segmenter({ dictionary: Array(1) }), 'ERR_KH_OPTIONS')
    const segmenter = new Segmenter()
    assertThrowsCode(() => segmenter.addWords(Array(1)), 'ERR_KH_WORDS')
    assert.deepEqual(tokenTexts(segmenter.segment(CUSTOM_WORD)), baseline)
  } finally {
    delete Array.prototype[0]
  }
})

test('every frequency key is validated, including colliding lone surrogates', () => {
  assertThrowsCode(() => new Segmenter().addWords({ ['\uD800']: false }), 'ERR_KH_WORDS')
  assertThrowsCode(() => new Segmenter({ dictionary: { ['\uD800']: false } }), 'ERR_KH_OPTIONS')
  assertThrowsCode(
    () => new Segmenter().addWords({ ['\uD800']: 0, ['\uD801']: 500 }),
    'ERR_KH_WORDS',
  )

  const withSymbol = { [Symbol('x')]: 'ignored', [CUSTOM_WORD]: 50_000 }
  const segmenter = new Segmenter().addWords(withSymbol)
  assert.deepEqual(tokenTexts(segmenter.segment(CUSTOM_WORD)), [CUSTOM_WORD])
  const fromOptions = new Segmenter({ dictionary: withSymbol })
  assert.deepEqual(tokenTexts(fromOptions.segment(CUSTOM_WORD)), [CUSTOM_WORD])
})

test('brand checks still apply without process.getBuiltinModule', () => {
  const descriptor = Object.getOwnPropertyDescriptor(process, 'getBuiltinModule')
  try {
    delete process.getBuiltinModule
    assert.equal(process.getBuiltinModule, undefined)

    const foreign = runInNewContext("({ mode: 'viterbi' })")
    for (const options of [{}, Object.create(null), foreign]) {
      assert.deepEqual(segment('ខ្ញុំ', options), ['ខ្ញុំ'])
      assert.ok(new Segmenter(options) instanceof Segmenter)
    }
    for (const words of [{}, Object.create(null), runInNewContext('({})')]) {
      assert.ok(new Segmenter({ dictionary: words }) instanceof Segmenter)
      assert.ok(new Segmenter().addWords(words) instanceof Segmenter)
    }

    const nonPlain = [
      () => Object.setPrototypeOf(new Map(), null),
      () => Object.setPrototypeOf(new Set(), null),
      () => Object.setPrototypeOf(/x/, null),
      () => Object.setPrototypeOf(Object(1), null),
      () => Object.setPrototypeOf(new WeakRef({}), null),
      () => Object.setPrototypeOf(new FinalizationRegistry(() => {}), null),
      () => Object.setPrototypeOf(new Segmenter(), null),
      () => new Proxy(new Date(), {}),
      () => new Proxy({}, {}),
      () => new Proxy([], {}),
      () => Object.create(Object.create(null)),
    ]
    for (const make of nonPlain) {
      const label = String(make)
      assert.throws(() => segment('x', make()), { code: 'ERR_KH_OPTIONS' }, label)
      assert.throws(() => new Segmenter(make()), { code: 'ERR_KH_OPTIONS' }, label)
      assert.throws(() => new Segmenter({ dictionary: make() }), { code: 'ERR_KH_OPTIONS' }, label)
      assert.throws(() => new Segmenter().addWords(make()), { code: 'ERR_KH_WORDS' }, label)
    }
  } finally {
    if (descriptor) Object.defineProperty(process, 'getBuiltinModule', descriptor)
  }
  assert.equal(typeof process.getBuiltinModule, 'function')
})

test('Object.prototype.get accessor does not run during brand checks', () => {
  try {
    Object.defineProperty(Object.prototype, 'get', {
      configurable: true,
      get() {
        throw new Error('Object.prototype.get ran')
      },
    })

    assert.deepEqual(segment('ខ្ញុំ', {}), ['ខ្ញុំ'])
    assert.ok(new Segmenter({}) instanceof Segmenter)
    const fromDict = new Segmenter({ dictionary: { [CUSTOM_WORD]: 50_000 } })
    assert.deepEqual(tokenTexts(fromDict.segment(CUSTOM_WORD)), [CUSTOM_WORD])
    const fromAdd = new Segmenter().addWords({ [CUSTOM_WORD]: 50_000 })
    assert.deepEqual(tokenTexts(fromAdd.segment(CUSTOM_WORD)), [CUSTOM_WORD])
  } finally {
    delete Object.prototype.get
  }
})

test('own enumerable Symbol getters on frequency maps are never invoked', () => {
  const withSymbolGetter = Object.defineProperty({ [CUSTOM_WORD]: 50_000 }, Symbol('probe'), {
    enumerable: true,
    get() {
      throw new Error('symbol getter ran')
    },
  })

  const fromDict = new Segmenter({ dictionary: withSymbolGetter })
  assert.deepEqual(tokenTexts(fromDict.segment(CUSTOM_WORD)), [CUSTOM_WORD])
  const fromAdd = new Segmenter().addWords(withSymbolGetter)
  assert.deepEqual(tokenTexts(fromAdd.segment(CUSTOM_WORD)), [CUSTOM_WORD])
})
