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
 * Node 18-compatible well-formedness conversion.
 *
 * N-API exposes Rust `String` values, so a JavaScript string is encoded as
 * UTF-8 on the way in. Lone UTF-16 surrogates have no UTF-8 representation and
 * are replaced with U+FFFD before the core sees them. This helper reproduces
 * that conversion without relying on newer runtime methods, so tests also run
 * on Node 18.
 *
 * @param {string} text
 */
export function wellFormed(text) {
  let result = ''
  for (let index = 0; index < text.length; index += 1) {
    const code = text.charCodeAt(index)
    if (code >= 0xd800 && code <= 0xdbff) {
      const next = text.charCodeAt(index + 1)
      if (next >= 0xdc00 && next <= 0xdfff) {
        result += text[index] + text[index + 1]
        index += 1
      } else {
        result += '\uFFFD'
      }
    } else if (code >= 0xdc00 && code <= 0xdfff) {
      result += '\uFFFD'
    } else {
      result += text[index]
    }
  }
  return result
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
