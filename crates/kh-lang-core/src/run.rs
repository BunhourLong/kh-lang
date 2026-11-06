//! Script classification for Khmer vs. everything else.
//!
//! [`runs`] splits any input into maximal, contiguous, single-`[RunKind]`
//! byte ranges. Later tasks apply Khmer segmentation only to Khmer runs and
//! keep every other run (whitespace, punctuation, Latin, digits, emoji) as a
//! single token, which guarantees lossless tokenization.
//!
//! Classification considers a character Khmer when it belongs to the Khmer
//! block `U+1780..U+17FF` (consonants, vowels, digits, signs and Khmer
//! punctuation), the supplementary Khmer Symbols block `U+19E0..U+19FF`
//! (lunar dates), or is the zero-width non-joiner `U+200C`, which Khmer
//! spelling uses to prevent ligatures and which must not be orphaned into a
//! non-Khmer token. `U+200B` (ZWSP) is intentionally *not* Khmer and not
//! special-cased here — callers strip it with
//! [`normalize`](crate::normalize) first.

use std::ops::Range;

/// The script class of a [`Run`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunKind {
    /// Khmer script: `U+1780..U+17FF`, Khmer symbols `U+19E0..U+19FF`, and
    /// the zero-width non-joiner `U+200C`.
    Khmer,
    /// Anything else: whitespace, punctuation, Latin letters, digits, emoji,
    /// other scripts.
    Other,
}

impl RunKind {
    /// Classifies a single character.
    #[must_use]
    pub fn of(c: char) -> Self {
        if ('\u{1780}'..='\u{17FF}').contains(&c)
            || ('\u{19E0}'..='\u{19FF}').contains(&c)
            || c == '\u{200C}'
        {
            Self::Khmer
        } else {
            Self::Other
        }
    }
}

/// A maximal contiguous stretch of characters sharing one [`RunKind`].
///
/// `byte_range` addresses the input string in UTF-8 byte offsets;
/// `input[byte_range]` reproduces exactly the characters of this run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    /// The script class shared by every character in the run.
    pub kind: RunKind,
    /// Byte range of the run within the input string.
    pub byte_range: Range<usize>,
}

/// Splits `s` into maximal contiguous [`Run`]s of single-`[RunKind]` class.
///
/// Runs appear in source order, cover the whole input, and concatenate back
/// to exactly `s`. Empty input yields no runs; the function never panics.
///
/// # Examples
///
/// ```
/// use kh_lang_core::{RunKind, runs};
///
/// let runs = runs("ខ្ញុំabc 1");
/// assert_eq!(runs.len(), 2);
/// assert_eq!(runs[0].kind, RunKind::Khmer);
/// assert_eq!(runs[1].kind, RunKind::Other);
/// ```
#[must_use]
pub fn runs(s: &str) -> Vec<Run> {
    let mut out = Vec::new();
    let mut kind: Option<RunKind> = None;
    let mut start = 0;
    let mut end = 0;
    for (i, c) in s.char_indices() {
        let next_kind = RunKind::of(c);
        let next_end = i + c.len_utf8();
        match kind {
            Some(this_kind) if this_kind == next_kind => end = next_end,
            Some(this_kind) => {
                out.push(Run {
                    kind: this_kind,
                    byte_range: start..end,
                });
                start = i;
                end = next_end;
                kind = Some(next_kind);
            }
            None => {
                start = i;
                end = next_end;
                kind = Some(next_kind);
            }
        }
    }
    if let Some(kind) = kind {
        out.push(Run {
            kind,
            byte_range: start..end,
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{Run, RunKind, runs};

    #[test]
    fn mixed_khmer_and_latin() {
        // "ខ្ញុំ" = 5 chars × 3 bytes = 15 bytes; "abc 12" = 6 bytes.
        let out = runs("ខ្ញុំabc 12");
        assert_eq!(
            out,
            vec![
                Run {
                    kind: RunKind::Khmer,
                    byte_range: 0..15
                },
                Run {
                    kind: RunKind::Other,
                    byte_range: 15..21
                },
            ]
        );
    }

    #[test]
    fn zwnj_counts_as_khmer() {
        let out = runs("ក\u{200C}x");
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].kind, RunKind::Khmer);
        assert_eq!(out[0].byte_range, 0..6); // ក = 3 bytes, ZWNJ = 3 bytes
    }

    #[test]
    fn lunar_date_symbols_count_as_khmer() {
        let out = runs("\u{19E0}");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].kind, RunKind::Khmer);
    }

    #[test]
    fn zwsp_is_other() {
        let out = runs("ក\u{200B}");
        assert_eq!(out.len(), 2);
        assert_eq!(out[1].kind, RunKind::Other);
    }

    #[test]
    fn khmer_runs_merge_and_alternate() {
        // ក(3) + ា(3) Khmer; a Other; ខ Khmer.
        let out = runs("កាaខ");
        let kinds: Vec<RunKind> = out.iter().map(|r| r.kind).collect();
        assert_eq!(kinds, vec![RunKind::Khmer, RunKind::Other, RunKind::Khmer]);
        assert_eq!(out[0].byte_range, 0..6);
        assert_eq!(out[1].byte_range, 6..7);
        assert_eq!(out[2].byte_range, 7..10);
    }

    #[test]
    fn empty_input_yields_no_runs() {
        assert!(runs("").is_empty());
    }

    #[test]
    fn non_khmer_scripts_are_other() {
        let out = runs("ไทย \u{1F600}.");
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].kind, RunKind::Other);
    }
}
