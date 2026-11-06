import assert from 'node:assert/strict'

export const ZWSP = '\u200B'
export const BOM = '\uFEFF'

/** @param {string} text */
export function stripZwsp(text) {
  return text.replaceAll(ZWSP, '')
}

/** @param {string} text */
export function stripBomAndZwsp(text) {
  return text.replaceAll(ZWSP, '').replaceAll(BOM, '')
}

/** @param {string} text */
export function countZwsp(text) {
  return [...text].filter((char) => char === ZWSP).length
}

/**
 * N-API encodes JavaScript strings as UTF-8, so lone UTF-16 surrogates reach
 * the core as U+FFFD; this mirrors that conversion.
 *
 * @param {string} text
 */
export function wellFormed(text) {
  return text.toWellFormed()
}

/** @param {number} code */
export function isKhmerCodeUnit(code) {
  return code >= 0x1780 && code <= 0x17ff
}

/**
 * Asserts the token invariants shared by `Segmenter#segment` and `#split`.
 *
 * @param {string} text
 * @param {{ text: string, start: number, end: number }[]} tokens
 */
export function assertTokenInvariants(text, tokens) {
  let cursor = 0
  for (const token of tokens) {
    assert.equal(typeof token.text, 'string')
    assert.equal(text.slice(token.start, token.end), token.text)
    assert.equal(token.end - token.start, token.text.length)
    assert.equal(token.start, cursor)
    cursor = token.end
  }
  assert.equal(cursor, text.length)
}

/**
 * @param {{ text: string }[]} tokens
 */
export function tokenTexts(tokens) {
  return tokens.map((token) => token.text)
}
