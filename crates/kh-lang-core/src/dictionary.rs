//! A frequency-annotated word dictionary for Khmer segmentation.
//!
//! [`Dictionary`] maps a word (`&str`) to its frequency (`u32`). The embedded
//! dictionary ([`Dictionary::embedded`]) is built from `data/khmer.lbdict.txt`
//! (SIL Khmer dictionary data, MIT — see `licenses/`), embedded with
//! `include_str!` and parsed lazily into a process-wide singleton guarded by a
//! [`std::sync::OnceLock`] so repeated calls never re-parse 33k lines.
//!
//! # Parsing (`word\tfreq` lines)
//!
//! [`Dictionary::from_text`] is deliberately lenient — malformed input never
//! aborts, it just contributes fewer entries:
//!
//! - a line is `word`, optionally followed by a tab and a decimal frequency;
//! - a line with **no** frequency column, or with a column that does not parse
//!   as `u32`, receives the default frequency [`DEFAULT_FREQ`] (`10_000`) —
//!   e.g. trailing build-tool marker lines in pre-filtered dictionaries;
//! - only completely blank (or tab-only) lines are skipped;
//! - the upstream build pipeline strips entries containing ASCII or spaces, so
//!   such words are not filtered here (the parser stays lenient: a stray
//!   purely-ASCII line would simply be inserted as-is).
//!
//! Frequencies feed only the viterbi scorer, which compares log-probabilities
//! scaled by the total corpus size; see [`crate::matcher`].

use std::collections::HashMap;
use std::sync::OnceLock;

/// Frequency assigned to dictionary lines that carry no usable frequency.
pub const DEFAULT_FREQ: u32 = 10_000;

/// A word → frequency map for segmentation.
///
/// Insert/lookup are by `&str` slices; matching later slices whole bytes out
/// of the input, so the crate never needs to allocate a lookup key except
/// when inserting.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Dictionary {
    words: HashMap<String, u32>,
    total: u64,
    max_word_len: usize,
}

impl Dictionary {
    /// A fresh, empty dictionary.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The embedded `data/khmer.lbdict.txt` dictionary, parsed once lazily
    /// and shared process-wide.
    ///
    /// # Examples
    ///
    /// ```
    /// use kh_lang_core::dictionary::Dictionary;
    ///
    /// assert!(Dictionary::embedded().len() > 30_000);
    /// assert!(Dictionary::embedded().contains("ខ្ញុំ"));
    /// ```
    #[must_use]
    pub fn embedded() -> &'static Dictionary {
        static EMBEDDED: OnceLock<Dictionary> = OnceLock::new();
        EMBEDDED
            .get_or_init(|| Dictionary::from_text(include_str!("../../../data/khmer.lbdict.txt")))
    }

    /// Parses `word\tfreq` lines (see the [module docs](self) for the exact,
    /// lenient rules) into a new [`Dictionary`].
    #[must_use]
    pub fn from_text(text: &str) -> Dictionary {
        let mut dict = Dictionary::new();
        for line in text.lines() {
            // Trim only the end: a leading tab means "empty word column" and
            // must survive until `split_once` so the line can be skipped.
            let line = line.trim_end();
            if line.trim().is_empty() {
                continue;
            }
            let (word, freq) = match line.split_once('\t') {
                Some((word, freq)) => (word, freq.trim().parse::<u32>().ok()),
                None => (line, None),
            };
            let word = word.trim();
            if word.is_empty() {
                continue;
            }
            dict.insert(word, freq.unwrap_or(DEFAULT_FREQ));
        }
        dict
    }

    /// Inserts (or replaces) one word with the given frequency.
    ///
    /// Bookkeeping of the frequency total and the maximum word length is
    /// maintained automatically for the viterbi scorer.
    pub fn insert(&mut self, word: &str, freq: u32) {
        match self.words.insert(String::from(word), freq) {
            Some(prev) => {
                self.total = self
                    .total
                    .saturating_sub(u64::from(prev))
                    .saturating_add(u64::from(freq));
            }
            None => {
                self.total = self.total.saturating_add(u64::from(freq));
                if word.len() > self.max_word_len {
                    self.max_word_len = word.len();
                }
            }
        }
    }

    /// Adds a word with the default frequency [`DEFAULT_FREQ`].
    pub fn add(&mut self, word: &str) {
        self.insert(word, DEFAULT_FREQ);
    }

    /// Adds every word with the default frequency.
    pub fn add_words(&mut self, words: &[&str]) {
        for &word in words {
            self.add(word);
        }
    }

    /// Returns `true` when `word` is in the dictionary.
    #[must_use]
    pub fn contains(&self, word: &str) -> bool {
        self.words.contains_key(word)
    }

    /// The frequency of `word`, or `None` when it is absent.
    #[must_use]
    pub fn freq(&self, word: &str) -> Option<u32> {
        self.words.get(word).copied()
    }

    /// Number of distinct words.
    #[must_use]
    pub fn len(&self) -> usize {
        self.words.len()
    }

    /// Whether the dictionary has no words.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }

    /// Sum of all stored frequencies as `f64`, the corpus size used to turn
    /// frequencies into log-probabilities in [`crate::matcher`].
    ///
    /// The sum is saturating (it can never overflow) and clamped to at least
    /// `1.0`, so callers may take its logarithm unconditionally. An empty
    /// dictionary therefore reports `1.0`, not `0.0`.
    #[must_use]
    pub fn total_freq(&self) -> f64 {
        self.total.max(1) as f64
    }

    /// Length in bytes of the longest stored word; matcher candidate spans
    /// longer than this can never hit the dictionary.
    #[must_use]
    pub(crate) fn max_word_len(&self) -> usize {
        self.max_word_len
    }
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_FREQ, Dictionary};

    #[test]
    fn new_is_empty() {
        let d = Dictionary::new();
        assert!(d.is_empty());
        assert_eq!(d.len(), 0);
        assert_eq!(d.total_freq(), 1.0);
        assert_eq!(d.max_word_len(), 0);
    }

    #[test]
    fn inserts_and_lookups() {
        let mut d = Dictionary::new();
        d.insert("ខ្ញុំ", 30_168);
        assert!(d.contains("ខ្ញុំ"));
        assert_eq!(d.freq("ខ្ញុំ"), Some(30_168));
        assert!(!d.contains("យើង"));
        assert_eq!(d.freq("យើង"), None);
        // Replacing the frequency keeps totals consistent.
        d.insert("ខ្ញុំ", 10);
        assert_eq!(d.total_freq(), 10.0);
        d.add("យើង");
        assert_eq!(d.freq("យើង"), Some(DEFAULT_FREQ));
        assert_eq!(d.total_freq(), 10.0 + f64::from(DEFAULT_FREQ));
    }

    #[test]
    fn add_words_bulk() {
        let mut d = Dictionary::new();
        d.add_words(&["ក", "ខ្ញុំ", "abc"]);
        assert_eq!(d.len(), 3);
        assert!(d.contains("abc"));
        assert_eq!(d.max_word_len(), "ខ្ញុំ".len());
    }

    #[test]
    fn parses_word_freq_lines() {
        let d = Dictionary::from_text("កា\t100\nខៃ\t2");
        assert_eq!(d.freq("កា"), Some(100));
        assert_eq!(d.freq("ខៃ"), Some(2));
        assert_eq!(d.total_freq(), 102.0);
        assert_eq!(d.max_word_len(), "ខៃ".len());
    }

    #[test]
    fn lines_without_number_get_default_freq() {
        // No tab at all, and a tab with a non-numeric column: both get the
        // default frequency instead of aborting.
        let d = Dictionary::from_text("កំណត់\nខ្ញុំ\tnot-a-number\n");
        assert_eq!(d.freq("កំណត់"), Some(DEFAULT_FREQ));
        assert_eq!(d.freq("ខ្ញុំ"), Some(DEFAULT_FREQ));
        assert_eq!(d.len(), 2);
    }

    #[test]
    fn malformed_and_blank_lines_are_skipped() {
        let d = Dictionary::from_text("\n\t\t  \nកា\t7\n\trest\n  \n");
        assert_eq!(d.freq("កា"), Some(7));
        // A line whose word column is empty or blank never yields an entry.
        assert!(!d.contains("rest"));
        assert!(!d.contains("  "));
        assert_eq!(d.len(), 1);
    }

    #[test]
    fn crlf_is_handled() {
        let d = Dictionary::from_text("កា\t7\r\nខៃ\t3\r\n");
        assert_eq!(d.freq("កា"), Some(7));
        assert_eq!(d.freq("ខៃ"), Some(3));
        assert_eq!(d.len(), 2);
    }

    #[test]
    fn embedded_dictionary_is_loaded() {
        let d = Dictionary::embedded();
        assert!(d.len() > 30_000);
        for word in ["ខ្ញុំ", "ស្រលាញ់", "ភ្នំពេញ", "កម្ពុជា"]
        {
            assert!(d.contains(word), "missing {word}");
            assert!(d.freq(word).is_some_and(|f| f > 0));
        }
        assert_eq!(d.freq("ខ្ញុំ"), Some(30_168));
        assert_eq!(d.freq("ស្រលាញ់"), Some(557));
    }

    #[test]
    fn embedded_returns_same_instance() {
        assert!(std::ptr::eq(Dictionary::embedded(), Dictionary::embedded()));
    }
}
