//! Lossless Khmer word segmentation built on the crate's runs, dictionary and
//! matchers.
//!
//! [`Segmenter::segment`] splits any input into [`Token`]s that concatenate
//! back to the exact input — including invisible characters such as ZWSP
//! (`U+200B`) and BOM (`U+FEFF`). It deliberately does **not** call
//! [`normalize`](crate::normalize): that API removes those characters, so
//! callers who want cleanup must run it explicitly before segmenting.
//!
//! The pipeline is:
//!
//! 1. [`runs`](crate::run::runs) splits the input into Khmer, Khmer-digit and
//!    non-Khmer stretches.
//! 2. Every non-Khmer stretch becomes exactly one [`Token`] (whitespace,
//!    Latin, digits, emoji, ZWSP, BOM, ...), and so does every stretch of
//!    Khmer digits `U+17E0..=U+17E9`, so a number is never split.
//! 3. Every Khmer stretch is segmented with [`bimm`] or [`viterbi`] depending
//!    on the configured [`Mode`].
//!
//! Offsets are UTF-16 code units (what JavaScript strings use), so the native
//! bindings can hand them to JS unchanged.

use std::borrow::Cow;
use std::ops::Range;

use crate::cluster::is_khmer_mark;
use crate::dictionary::Dictionary;
use crate::matcher::{bimm, viterbi};
use crate::run::{RunKind, is_khmer_digit, runs};

/// Segmentation algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// Bidirectional maximal matching (default).
    ///
    /// A forward greedy pass and a backward greedy pass run over the
    /// dictionary; the path with fewer tokens wins and ties prefer the
    /// forward path.
    #[default]
    Bimm,
    /// Frequency-scored Viterbi dynamic programming.
    ///
    /// Dictionary frequencies become log-probabilities; out-of-vocabulary
    /// clusters receive a small constant penalty and are emitted one cluster
    /// at a time rather than discarded.
    Viterbi,
}

/// One segment of the original input.
///
/// `text` is the exact slice of the input this token covers. `start` and
/// `end` are **UTF-16 code-unit** offsets into the original input — the
/// offsets JavaScript's `String` uses — so `end - start` equals
/// `text.encode_utf16().count()` for any input below `u32::MAX` UTF-16 code
/// units, which covers every JavaScript-reachable string. Only an input
/// beyond that bound (unreachable from JS) saturates the offsets; the length
/// and contiguity guarantees hold up to that point.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// The token's characters, copied from the input.
    pub text: String,
    /// UTF-16 code-unit offset of the token's first code unit in the input.
    pub start: u32,
    /// UTF-16 code-unit offset one past the token's last code unit.
    pub end: u32,
}

/// A reusable segmenter with a fixed dictionary and [`Mode`].
///
/// [`Segmenter::new`] borrows the process-wide embedded dictionary without
/// copying it; the base table is cloned only when a [`SegmenterBuilder`] adds
/// custom words or installs a replacement dictionary.
#[derive(Debug, Clone)]
pub struct Segmenter {
    dictionary: Cow<'static, Dictionary>,
    mode: Mode,
}

impl Default for Segmenter {
    fn default() -> Self {
        Self::new()
    }
}

impl Segmenter {
    /// Creates a segmenter backed by the embedded dictionary in
    /// [`Mode::Bimm`], without copying the 33k-entry base table.
    ///
    /// # Examples
    ///
    /// ```
    /// use kh_lang_core::{Mode, Segmenter};
    ///
    /// assert_eq!(Segmenter::new().mode(), Mode::Bimm);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            dictionary: Cow::Borrowed(Dictionary::embedded()),
            mode: Mode::default(),
        }
    }

    /// Starts a [`SegmenterBuilder`] with the same defaults as
    /// [`Segmenter::new`].
    ///
    /// # Examples
    ///
    /// ```
    /// use kh_lang_core::{Mode, Segmenter};
    ///
    /// let segmenter = Segmenter::builder().mode(Mode::Viterbi).build();
    /// assert_eq!(segmenter.mode(), Mode::Viterbi);
    /// ```
    #[must_use]
    pub fn builder() -> SegmenterBuilder {
        SegmenterBuilder::default()
    }

    /// The configured segmentation algorithm.
    #[must_use]
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Segments `text` into contiguous, lossless [`Token`]s.
    ///
    /// Non-Khmer stretches (whitespace, punctuation, Latin, digits, emoji,
    /// ZWSP, BOM, ...) and contiguous Khmer digits each become one token;
    /// Khmer stretches are segmented with the configured [`Mode`]. Offsets are UTF-16 code units and are
    /// contiguous across the whole output: the first token starts at `0`,
    /// every following token starts where the previous one ended, and the
    /// last token ends at `text.encode_utf16().count()`. These guarantees are
    /// exact for inputs up to `u32::MAX` UTF-16 code units, which covers every
    /// JavaScript-reachable string; only beyond that bound do the offsets
    /// saturate. Empty input yields an empty vector, and the function never
    /// panics on arbitrary input.
    ///
    /// [`normalize`](crate::normalize) is intentionally **not** applied:
    /// it drops ZWSP and BOM, which would break losslessness. Run it
    /// explicitly first if that cleanup is desired.
    ///
    /// # Examples
    ///
    /// ```
    /// use kh_lang_core::Segmenter;
    ///
    /// let tokens = Segmenter::new().segment("ខ្ញុំស្រលាញ់ភ្នំពេញ");
    /// let words: Vec<&str> = tokens.iter().map(|token| token.text.as_str()).collect();
    /// assert_eq!(words, ["ខ្ញុំ", "ស្រលាញ់", "ភ្នំពេញ"]);
    /// ```
    #[must_use]
    pub fn segment(&self, text: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut utf16_offset = 0u32;
        for run in runs(text) {
            let run_text = &text[run.byte_range];
            match run.kind {
                RunKind::Other | RunKind::KhmerDigit => {
                    utf16_offset = push_token(&mut tokens, run_text, utf16_offset);
                }
                RunKind::Khmer => {
                    let ranges = match self.mode {
                        Mode::Bimm => bimm(&self.dictionary, run_text),
                        Mode::Viterbi => viterbi(&self.dictionary, run_text),
                    };
                    for range in ranges {
                        utf16_offset = push_token(&mut tokens, &run_text[range], utf16_offset);
                    }
                }
            }
        }
        tokens
    }

    /// Segments `text` and inserts U+200B ZWSP between adjacent tokens whose
    /// touching characters are both Khmer (letters, signs or Khmer digits).
    ///
    /// Nothing is inserted at the string edges, before or after empty text, or
    /// next to existing ZWSP, whitespace, Latin text or any other non-Khmer
    /// character, because those boundary characters belong to non-Khmer
    /// tokens. A ZWSP is also never inserted after a coeng (`U+17D2`), before
    /// a coeng or a combining mark, before the closing signs `។ ៕ ៖ ៗ ៚ ៛`
    /// (token boundaries after them still allow ZWSP), nor between two Khmer digits
    /// (`U+17E0..=U+17E9`), which would split a number. Removing the ZWSP this
    /// call inserted restores `text`; pre-existing U+200B in the input are
    /// preserved. Chunks left joined by a suppressed boundary (or in Viterbi
    /// mode, where rounding ties can shift without trailing context) are
    /// re-segmented to a fixed point so the call is strictly idempotent.
    ///
    /// # Examples
    ///
    /// ```
    /// use kh_lang_core::Segmenter;
    ///
    /// let segmenter = Segmenter::new();
    /// let spaced = segmenter.insert_zwsp("ខ្ញុំស្រលាញ់");
    /// assert_eq!(spaced, "ខ្ញុំ\u{200B}ស្រលាញ់");
    /// assert_eq!(spaced.replace('\u{200B}', ""), "ខ្ញុំស្រលាញ់");
    /// assert_eq!(segmenter.insert_zwsp(&spaced), spaced);
    /// ```
    #[must_use]
    pub fn insert_zwsp(&self, text: &str) -> String {
        let mut out = String::with_capacity(text.len());
        let mut previous = "";
        for run in runs(text) {
            let Some(run_text) = text.get(run.byte_range) else {
                continue;
            };
            if zwsp_allowed(previous, run_text) {
                out.push('\u{200B}');
            }
            match run.kind {
                RunKind::Other | RunKind::KhmerDigit => out.push_str(run_text),
                RunKind::Khmer => self.write_khmer_run_with_zwsp(run_text, &mut out),
            }
            previous = run_text;
        }
        out
    }

    /// Writes one Khmer run to `out` with ZWSP between its words.
    ///
    /// Tokens joined by a suppressed boundary form one chunk. Multi-token
    /// chunks (and any Viterbi chunk, whose floating-point rounding ties can
    /// shift once trailing context is removed) are re-segmented on their own
    /// until no chunk splits further, making the call strictly idempotent.
    /// Every re-queued span is strictly shorter, so this ends.
    fn write_khmer_run_with_zwsp(&self, run_text: &str, out: &mut String) {
        let mut stack = vec![(0..run_text.len(), true)];
        let mut first_piece = true;
        while let Some((span, needs_split)) = stack.pop() {
            let Some(slice) = run_text.get(span.clone()) else {
                continue;
            };
            let mut chunks: Vec<(Range<usize>, usize)> = Vec::new();
            if needs_split {
                let ranges = match self.mode {
                    Mode::Bimm => bimm(&self.dictionary, slice),
                    Mode::Viterbi => viterbi(&self.dictionary, slice),
                };
                for range in ranges {
                    if let Some(last) = chunks.last_mut()
                        && let (Some(prev), Some(next)) =
                            (slice.get(last.0.clone()), slice.get(range.clone()))
                        && !zwsp_allowed(prev, next)
                    {
                        last.0.end = range.end;
                        last.1 += 1;
                    } else {
                        chunks.push((range, 1));
                    }
                }
            }
            if chunks.len() <= 1 {
                if !first_piece {
                    out.push('\u{200B}');
                }
                out.push_str(slice);
                first_piece = false;
                continue;
            }
            for (chunk, token_count) in chunks.into_iter().rev() {
                stack.push((
                    span.start + chunk.start..span.start + chunk.end,
                    token_count > 1 || self.mode == Mode::Viterbi,
                ));
            }
        }
    }
}

/// Appends one token for `piece` at `offset`, returning the next offset.
///
/// UTF-16 lengths and offsets saturate at [`u32::MAX`] instead of panicking on
/// inputs too large to address with 32-bit offsets.
fn push_token(tokens: &mut Vec<Token>, piece: &str, offset: u32) -> u32 {
    let end = offset.saturating_add(utf16_len(piece));
    tokens.push(Token {
        text: piece.to_owned(),
        start: offset,
        end,
    });
    end
}

/// UTF-16 code-unit length of `s`, saturating at [`u32::MAX`].
fn utf16_len(s: &str) -> u32 {
    u32::try_from(s.encode_utf16().count()).unwrap_or(u32::MAX)
}

/// Khmer signs that close the preceding text and take no ZWSP before them:
/// khan `។`, bariyoosan `៕`, camnuc pii kuuh `៖`, lek too `ៗ`, koomuut `៚` and
/// the riel sign `៛`.
fn is_khmer_closing_punct(c: char) -> bool {
    matches!(
        c,
        '\u{17D4}' | '\u{17D5}' | '\u{17D6}' | '\u{17D7}' | '\u{17DA}' | '\u{17DB}'
    )
}

/// Whether a ZWSP may be inserted between the adjacent tokens `left` and
/// `right`.
///
/// The touching characters must both be Khmer letters, signs or digits. The
/// left character must not be a coeng (`U+17D2`), which binds to the next base
/// character; the right character must not be a coeng, a combining mark or a
/// closing sign (see `is_khmer_closing_punct`); and the boundary must not fall
/// between two Khmer digits, which belong to one number.
fn zwsp_allowed(left: &str, right: &str) -> bool {
    let (Some(last), Some(first)) = (left.chars().next_back(), right.chars().next()) else {
        return false;
    };
    matches!(RunKind::of(last), RunKind::Khmer | RunKind::KhmerDigit)
        && matches!(RunKind::of(first), RunKind::Khmer | RunKind::KhmerDigit)
        && last != '\u{17D2}'
        && first != '\u{17D2}'
        && !is_khmer_mark(first)
        && !is_khmer_closing_punct(first)
        && !(is_khmer_digit(last) && is_khmer_digit(first))
}

/// Builder for a [`Segmenter`] with a custom mode, dictionary and extra words.
///
/// The defaults match [`Segmenter::new`]: the embedded dictionary in
/// [`Mode::Bimm`]. A replacement dictionary passed to
/// [`dictionary`](Self::dictionary) becomes the base table; words added with
/// [`add_word`](Self::add_word) or [`add_words`](Self::add_words) are layered
/// on top of whichever base table is selected.
#[derive(Debug, Clone, Default)]
pub struct SegmenterBuilder {
    mode: Mode,
    dictionary: Option<Dictionary>,
    extras: Vec<String>,
}

impl SegmenterBuilder {
    /// Selects the segmentation algorithm.
    #[must_use]
    pub fn mode(mut self, mode: Mode) -> Self {
        self.mode = mode;
        self
    }

    /// Replaces the base dictionary table.
    ///
    /// Calling this multiple times keeps only the last table. Extra words are
    /// still applied on top of it at [`build`](Self::build) time.
    #[must_use]
    pub fn dictionary(mut self, dictionary: Dictionary) -> Self {
        self.dictionary = Some(dictionary);
        self
    }

    /// Adds one extra word with the default dictionary frequency.
    ///
    /// Extras are applied on top of the selected base table; they never
    /// replace it.
    #[must_use]
    pub fn add_word(mut self, word: &str) -> Self {
        self.extras.push(word.to_owned());
        self
    }

    /// Adds every word in `words` with the default dictionary frequency.
    ///
    /// Extras are applied on top of the selected base table; they never
    /// replace it.
    #[must_use]
    pub fn add_words(mut self, words: &[&str]) -> Self {
        for &word in words {
            self.extras.push(word.to_owned());
        }
        self
    }

    /// Builds the [`Segmenter`].
    ///
    /// The embedded dictionary stays borrowed unless a replacement table or
    /// extra words require an owned copy.
    #[must_use]
    pub fn build(self) -> Segmenter {
        let mut dictionary = match self.dictionary {
            Some(dictionary) => Cow::Owned(dictionary),
            None => Cow::Borrowed(Dictionary::embedded()),
        };
        if !self.extras.is_empty() {
            let dictionary = dictionary.to_mut();
            for word in &self.extras {
                dictionary.add(word);
            }
        }
        Segmenter {
            dictionary,
            mode: self.mode,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Mode, Segmenter, Token, zwsp_allowed};
    use crate::dictionary::Dictionary;
    use crate::run::RunKind;

    fn token_texts(tokens: &[Token]) -> Vec<&str> {
        tokens.iter().map(|token| token.text.as_str()).collect()
    }

    /// Asserts that tokens are non-empty, contiguous, reproduce `text`
    /// exactly, and that every offset pair addresses the token's own UTF-16
    /// code units in the input.
    fn assert_lossless(text: &str, tokens: &[Token]) {
        let utf16: Vec<u16> = text.encode_utf16().collect();
        let mut cursor = 0u32;
        let mut flat = String::new();
        for token in tokens {
            assert!(!token.text.is_empty(), "empty token in {tokens:?}");
            assert_eq!(token.start, cursor, "tokens must be contiguous: {tokens:?}");
            assert!(token.start < token.end, "empty range in {tokens:?}");
            let expected: Vec<u16> = token.text.encode_utf16().collect();
            assert_eq!(
                usize::try_from(token.end - token.start).unwrap(),
                expected.len(),
                "range length must match the token's UTF-16 length"
            );
            let start = token.start as usize;
            let end = token.end as usize;
            assert!(end <= utf16.len(), "token past the input end: {tokens:?}");
            assert_eq!(&utf16[start..end], expected.as_slice());
            cursor = token.end;
            flat.push_str(&token.text);
        }
        assert_eq!(flat, text, "tokens must reproduce the input");
        assert_eq!(cursor as usize, utf16.len(), "tokens must cover the input");
    }

    #[test]
    fn default_segments_known_sentence() {
        let text = "ខ្ញុំស្រលាញ់ភ្នំពេញ";
        let tokens = Segmenter::new().segment(text);
        assert_eq!(token_texts(&tokens), vec!["ខ្ញុំ", "ស្រលាញ់", "ភ្នំពេញ"]);
        assert!(tokens.iter().all(|token| !token.text.is_empty()));
        assert_lossless(text, &tokens);
    }

    #[test]
    fn mixed_input_keeps_other_run_as_one_token() {
        let text = "ខ្ញុំ Hello, 123! 🙂 ភ្នំពេញ";
        let tokens = Segmenter::new().segment(text);
        assert_lossless(text, &tokens);

        let other: Vec<&Token> = tokens
            .iter()
            .filter(|token| token.text.contains("Hello"))
            .collect();
        assert_eq!(other.len(), 1, "the whole non-Khmer stretch is one token");
        assert_eq!(other[0].text, " Hello, 123! 🙂 ");
        assert_eq!(
            RunKind::of(other[0].text.chars().next().unwrap()),
            RunKind::Other
        );
        assert_eq!(
            RunKind::of(other[0].text.chars().next_back().unwrap()),
            RunKind::Other
        );
    }

    #[test]
    fn zwsp_and_bom_are_preserved_as_other_tokens() {
        let text = "ខ្ញុំ\u{200B}ស្រលាញ់\u{FEFF}ភ្នំពេញ";
        let tokens = Segmenter::new().segment(text);
        assert_lossless(text, &tokens);
        let texts = token_texts(&tokens);
        assert!(texts.contains(&"\u{200B}"), "ZWSP must survive: {texts:?}");
        assert!(texts.contains(&"\u{FEFF}"), "BOM must survive: {texts:?}");
    }

    #[test]
    fn viterbi_is_lossless_for_mixed_and_invisible_inputs() {
        let segmenter = Segmenter::builder().mode(Mode::Viterbi).build();
        assert_eq!(segmenter.mode(), Mode::Viterbi);
        for text in ["ខ្ញុំ Hello, 123! 🙂 ភ្នំពេញ", "ខ្ញុំ\u{200B}ស្រលាញ់\u{FEFF}ភ្នំពេញ"]
        {
            assert_lossless(text, &segmenter.segment(text));
        }
    }

    #[test]
    fn builder_extra_word_is_matched_as_one_token() {
        let text = "ខ្ញុំស្រលាញ់";
        let segmenter = Segmenter::builder().add_word(text).build();
        let tokens = segmenter.segment(text);
        assert_eq!(token_texts(&tokens), vec![text]);
        assert_lossless(text, &tokens);
    }

    #[test]
    fn builder_add_words_adds_every_extra() {
        let text = "ខ្ញុំស្រលាញ់";
        let segmenter = Segmenter::builder().add_words(&[text, "ភ្នំពេញ"]).build();
        let tokens = segmenter.segment(text);
        assert_eq!(token_texts(&tokens), vec![text]);
        assert_lossless(text, &tokens);
    }

    #[test]
    fn builder_dictionary_override_is_used_in_viterbi_mode() {
        let text = "ខ្ញុំភ្នំពេញ";
        // The embedded dictionary does not know this compound, so the whole
        // text becomes one token only when the override table is active.
        assert!(!Dictionary::embedded().contains(text));
        let segmenter = Segmenter::builder()
            .dictionary(Dictionary::from_text("ខ្ញុំភ្នំពេញ\t100\n"))
            .mode(Mode::Viterbi)
            .build();
        let tokens = segmenter.segment(text);
        assert_eq!(token_texts(&tokens), vec![text]);
        assert_lossless(text, &tokens);
    }

    #[test]
    fn builder_extra_word_layers_on_dictionary_override() {
        let text = "ខ្ញុំស្រលាញ់";
        let segmenter = Segmenter::builder()
            .dictionary(Dictionary::new())
            .add_word(text)
            .build();
        let tokens = segmenter.segment(text);
        assert_eq!(token_texts(&tokens), vec![text]);
        assert_lossless(text, &tokens);
    }

    #[test]
    fn builder_defaults_match_new() {
        let built = Segmenter::builder().build();
        assert_eq!(built.mode(), Mode::Bimm);
        let text = "ខ្ញុំស្រលាញ់ភ្នំពេញ";
        assert_eq!(built.segment(text), Segmenter::new().segment(text));
    }

    #[test]
    fn insert_zwsp_separates_khmer_words_and_is_idempotent() {
        let segmenter = Segmenter::new();
        let text = "ខ្ញុំស្រលាញ់ភ្នំពេញ";
        let spaced = segmenter.insert_zwsp(text);
        assert_eq!(spaced, "ខ្ញុំ\u{200B}ស្រលាញ់\u{200B}ភ្នំពេញ");
        assert_eq!(spaced.replace('\u{200B}', ""), text);
        assert_eq!(segmenter.insert_zwsp(&spaced), spaced);

        assert_eq!(segmenter.insert_zwsp("abc 123"), "abc 123");
        assert_eq!(segmenter.insert_zwsp("ក\u{200B}ខ"), "ក\u{200B}ខ");
    }

    #[test]
    fn zwsp_is_never_inserted_after_coeng() {
        // A coeng binds to the next base character, so a ZWSP there would
        // split the stack. Segmentation never emits a token boundary inside a
        // cluster, so the boundary predicate is checked directly.
        assert!(!zwsp_allowed("ក្", "ខ"));
        assert!(zwsp_allowed("ក", "ខ"));
    }

    #[test]
    fn zwsp_is_never_inserted_between_khmer_digits() {
        // An empty base table leaves the digits as separate cluster tokens,
        // so the old predicate would have split this number with a ZWSP.
        let segmenter = Segmenter::builder().dictionary(Dictionary::new()).build();
        assert_eq!(segmenter.insert_zwsp("១២"), "១២");
        assert_eq!(segmenter.insert_zwsp("១២៣"), "១២៣");
        // The exclusion is narrow: a digit next to a consonant still gets a
        // separator.
        assert_eq!(segmenter.insert_zwsp("១ក"), "១\u{200B}ក");
    }

    #[test]
    fn khmer_digits_and_oov_diacritics_segment_exactly() {
        let cases: &[(&str, &[&str])] = &[
            ("ឆ្នាំ២០២៦", &["ឆ្នាំ", "២០២៦"]),
            ("តម្លៃ១០០០៛", &["តម្លៃ", "១០០០", "៛"]),
            ("០១២៣៤៥៦៧៨៩", &["០១២៣៤៥៦៧៨៩"]),
            ("ទី១០", &["ទី", "១០"]),
            ("គ់", &["គ់"]),
            ("គ៉ា", &["គ៉ា"]),
            ("គ៍", &["គ៍"]),
            ("ក័ង", &["ក័", "ង"]),
            ("១\u{17D2}ក", &["១", "\u{17D2}ក"]),
            ("យូររបៀបថៃក្រម៉ា", &["យូរ", "របៀប", "ថៃ", "ក្រ", "ម៉ា"]),
        ];
        for mode in [Mode::Bimm, Mode::Viterbi] {
            let segmenter = Segmenter::builder().mode(mode).build();
            for &(text, expected) in cases {
                let tokens = segmenter.segment(text);
                assert_eq!(token_texts(&tokens), expected, "{text} ({mode:?})");
                assert_lossless(text, &tokens);
            }
        }
    }

    #[test]
    fn insert_zwsp_handles_closing_signs_digits_and_marks() {
        let segmenter = Segmenter::new();
        let cases = [
            ("ខ្ញុំស្រលាញ់ភ្នំពេញ។", "ខ្ញុំ\u{200B}ស្រលាញ់\u{200B}ភ្នំពេញ។"),
            ("តម្លៃ១០០០៛", "តម្លៃ\u{200B}១០០០៛"),
            ("ខ្ញុំ៕", "ខ្ញុំ៕"),
            ("ខ្ញុំ៖", "ខ្ញុំ៖"),
            ("ខ្ញុំៗ", "ខ្ញុំៗ"),
            ("ខ្ញុំ៚", "ខ្ញុំ៚"),
            ("ខ្ញុំ។ស្រលាញ់", "ខ្ញុំ។\u{200B}ស្រលាញ់"),
            ("ឆ្នាំ២០២៦", "ឆ្នាំ\u{200B}២០២៦"),
            ("១ក", "១\u{200B}ក"),
            ("១ា", "១ា"),
            ("១\u{17D2}ក", "១\u{17D2}ក"),
        ];
        for (text, expected) in cases {
            let spaced = segmenter.insert_zwsp(text);
            assert_eq!(spaced, expected, "{text}");
            assert_eq!(segmenter.insert_zwsp(&spaced), spaced, "{text} idempotent");
        }
        for sign in ["។", "៕", "៖", "ៗ", "៚", "៛"] {
            assert!(!zwsp_allowed("ខ្ញុំ", sign), "{sign}");
        }
        assert!(!zwsp_allowed("១", "២"));
    }

    #[test]
    fn insert_zwsp_is_idempotent_when_closing_sign_starts_a_token() {
        // BIMM picks ["ក", "។ខគ", "ឃងច"]; the suppressed boundary before "។ខគ"
        // used to leave "ក។ខគ" to be re-split on the next call.
        let words = ["ក។", "។ខគ", "ខគ", "ខគឃ", "ឃងច"];
        for mode in [Mode::Bimm, Mode::Viterbi] {
            for base in [None, Some(Dictionary::new())] {
                let mut builder = Segmenter::builder().mode(mode).add_words(&words);
                if let Some(dictionary) = base.clone() {
                    builder = builder.dictionary(dictionary);
                }
                let segmenter = builder.build();
                let spaced = segmenter.insert_zwsp("ក។ខគឃងច");
                assert_eq!(spaced, "ក។\u{200B}ខគ\u{200B}ឃងច", "{mode:?} {base:?}");
                assert!(!spaced.contains("\u{200B}។"));
                assert_eq!(segmenter.insert_zwsp(&spaced), spaced, "{mode:?} {base:?}");
            }
        }
    }

    #[test]
    fn insert_zwsp_viterbi_is_idempotent_across_rounding_ties() {
        // 70*70 / 2450 == 2 exactly, so floating-point rounding can flip the
        // tie on "ឝឞ" once the trailing OOV cluster "ឣ" is split off.
        let dict = Dictionary::from_text("ឝ\t70\nឞ\t70\nឝឞ\t2\nឤ\t2308\n");
        let segmenter = Segmenter::builder()
            .mode(Mode::Viterbi)
            .dictionary(dict)
            .build();
        let spaced = segmenter.insert_zwsp("ឝឞឣ");
        assert_eq!(spaced.replace('\u{200B}', ""), "ឝឞឣ");
        assert!(spaced.ends_with("\u{200B}ឣ"));
        assert_eq!(segmenter.insert_zwsp(&spaced), spaced);
    }

    #[test]
    fn empty_and_whitespace_inputs() {
        let segmenter = Segmenter::new();
        assert!(segmenter.segment("").is_empty());
        assert_eq!(segmenter.insert_zwsp(""), "");

        let text = "   \t\n";
        let tokens = segmenter.segment(text);
        assert_eq!(token_texts(&tokens), vec![text]);
        assert_lossless(text, &tokens);
        assert_eq!(segmenter.insert_zwsp(text), text);
    }

    #[test]
    fn pseudo_random_inputs_are_lossless() {
        // Fixed pool and fixed LCG seed: 200 deterministic cases, no crates.
        const POOL: &[&str] = &[
            "ក",
            "ខ",
            "គ",
            "ញ",
            "ស",
            "ព",
            "ភ",
            "ង",
            "\u{17D2}",
            "ា",
            "ិ",
            "ី",
            "ុ",
            "ំ",
            "ះ",
            "់",
            "a",
            "Z",
            " ",
            "\t",
            "1",
            "០",
            "១",
            "🙂",
            "😀",
            "\u{200B}",
            "\u{FEFF}",
            "\u{200C}",
            "\u{1F1F0}\u{1F1ED}",
        ];
        let mut state: u64 = 0x2545_F491_4F6C_DD1D;
        for _ in 0..200 {
            let len = (next(&mut state) % 24) as usize;
            let mut text = String::new();
            for _ in 0..len {
                let index = (next(&mut state) % POOL.len() as u64) as usize;
                text.push_str(POOL[index]);
            }

            for mode in [Mode::Bimm, Mode::Viterbi] {
                let segmenter = Segmenter::builder().mode(mode).build();
                assert_lossless(&text, &segmenter.segment(&text));
            }

            let segmenter = Segmenter::new();
            let spaced = segmenter.insert_zwsp(&text);
            // The brief's `replace('\u{200B}', "") == s` form only applies to
            // inputs without ZWSP; stripping every ZWSP also strips the
            // input's own ones. The general invariant is that no non-ZWSP
            // character changes and no input ZWSP is lost.
            assert_eq!(spaced.replace('\u{200B}', ""), text.replace('\u{200B}', ""));
            assert!(
                spaced.chars().filter(|&c| c == '\u{200B}').count()
                    >= text.chars().filter(|&c| c == '\u{200B}').count()
            );
            assert_eq!(segmenter.insert_zwsp(&spaced), spaced);
        }
    }

    fn next(state: &mut u64) -> u64 {
        *state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        *state >> 33
    }
}
