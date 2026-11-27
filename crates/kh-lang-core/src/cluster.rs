//! Grapheme-safe Khmer cluster iteration.
//!
//! Khmer stacks glyphs via the coeng ( subscribed-character) `U+17D2` placed
//! before a base consonant, followed by combining marks: inherent vowels
//! `U+17B4..U+17B5`, dependent vowels `U+17B6..U+17C5`, and the signs
//! `U+17C6..U+17D1` (nikhahit, reahmuk, register shifts, diacritics such as
//! bantoc, robat, kakabat, ahsda and viriam), plus bathamasat `U+17D3` and
//! atthacan `U+17DD`. A naive `str::chars()` iterator therefore slices through
//! the middle of a rendered glyph, breaking offset math in later matching and
//! slicing stages.
//!
//! [`clusters`] yields [`Cluster`]s with byte offsets into the input:
//!
//! - a coeng always binds to the character that follows it (their pair is kept
//!   together), transitively stacking consecutive `coeng + base` pairs;
//! - Khmer combining marks `U+17B4..U+17D1`, `U+17D3` and `U+17DD` attach to
//!   the cluster that precedes them (or form a lone cluster when nothing
//!   precedes);
//! - every other character — Khmer base characters, Latin letters, digits,
//!   whitespace, emoji — is its own single-character cluster.

/// A grapheme-safe cluster: a Khmer base character with any stacked coeng
/// sequences and combining marks, or a single non-clustered character.
///
/// `text` is the byte slice `input[byte_start..byte_end]` of the string the
/// iterator was created from; byte offsets are UTF-8 boundaries, so they are
/// safe to slice with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cluster<'a> {
    /// The cluster's characters, as a slice of the source string.
    pub text: &'a str,
    /// Byte offset of the cluster's first byte in the input string.
    pub byte_start: usize,
    /// Byte offset past the cluster's last byte in the input string.
    pub byte_end: usize,
}

const COENG: char = '\u{17D2}';

/// Khmer combining marks that attach to the preceding cluster: inherent vowels
/// `U+17B4..U+17B5`, dependent vowels `U+17B6..U+17C5`, signs `U+17C6..U+17D1`,
/// bathamasat `U+17D3` and atthacan `U+17DD`. Coeng `U+17D2` is excluded: it
/// binds forward. `U+17D4..U+17DC` (punctuation, currency) are not marks.
pub(crate) fn is_khmer_mark(c: char) -> bool {
    matches!(c, '\u{17B4}'..='\u{17D1}' | '\u{17D3}' | '\u{17DD}')
}

/// Iterator produced by [`clusters`].
///
/// Yields the grapheme-safe clusters of the input string in order; the
/// concatenation of all `text` slices equals the input string.
#[derive(Debug, Clone)]
pub struct ClusterIter<'a> {
    text: &'a str,
    pos: usize,
}

impl<'a> ClusterIter<'a> {
    fn new(text: &'a str) -> Self {
        Self { text, pos: 0 }
    }
}

impl<'a> Iterator for ClusterIter<'a> {
    type Item = Cluster<'a>;

    fn next(&mut self) -> Option<Cluster<'a>> {
        let s = self.text;
        let start = self.pos;
        let first = s.get(start..)?.chars().next()?;
        let mut end = start + first.len_utf8();
        // A coeng belongs to the character that follows it, even at the start
        // of a cluster, so that character (and any further stack) stays here.
        let mut after_coeng = first == COENG;
        while let Some(c) = s.get(end..).and_then(|rest| rest.chars().next()) {
            if after_coeng || is_khmer_mark(c) || c == COENG {
                end += c.len_utf8();
                after_coeng = c == COENG;
            } else {
                break;
            }
        }
        self.pos = end;
        Some(Cluster {
            text: s.get(start..end)?,
            byte_start: start,
            byte_end: end,
        })
    }
}

/// Iterates `s` as grapheme-safe clusters; see the [module documentation](self)
/// for the clustering rule.
///
/// Clusters concatenate back to exactly `s`, for arbitrary input, and the
/// function never panics.
///
/// # Examples
///
/// ```
/// use kh_lang_core::clusters;
///
/// let clusters: Vec<_> = clusters("ខ្ញុំ").map(|c| c.text).collect();
/// assert_eq!(clusters.join(""), "ខ្ញុំ");
/// ```
#[must_use]
pub fn clusters(s: &str) -> ClusterIter<'_> {
    ClusterIter::new(s)
}

#[cfg(test)]
mod tests {
    use super::clusters;

    fn texts(s: &str) -> Vec<&str> {
        clusters(s).map(|c| c.text).collect()
    }

    #[test]
    fn coeng_binds_to_next_base() {
        // ខ + ្ + ញ is one rendered glyph stack.
        assert_eq!(texts("ខ្ញ"), vec!["ខ្ញ"]);
    }

    #[test]
    fn khanhont_word_is_one_cluster() {
        // ខ ្ ញ ុ ំ — coeng pair plus two combining marks.
        assert_eq!(texts("ខ្ញុំ"), vec!["ខ្ញុំ"]);
    }

    #[test]
    fn latin_letters_are_single_clusters() {
        assert_eq!(texts("abc"), vec!["a", "b", "c"]);
        clusters("abc").for_each(|c| assert_eq!(c.byte_end - c.byte_start, 1));
    }

    #[test]
    fn mixed_text_offsets_are_byte_exact() {
        let s = "aខ្ញb";
        // 'a' = 1 byte; 'ខ្ញ' = ខ(3) + ្(3) + ញ(3) = 9 bytes; 'b' = 1 byte.
        assert_eq!(texts(s), vec!["a", "ខ្ញ", "b"]);
        let offsets: Vec<(usize, usize)> =
            clusters(s).map(|c| (c.byte_start, c.byte_end)).collect();
        assert_eq!(offsets, vec![(0, 1), (1, 10), (10, 11)]);
    }

    #[test]
    fn clusters_concatenate_back_exactly() {
        for s in ["ខ្ញុំ", "ស្រលាញ់", "ភ្នំពេញ", "abcខ្ញ123 \t\u{1F600}", ""]
        {
            assert_eq!(clusters(s).map(|c| c.text).collect::<Vec<_>>().join(""), s);
        }
    }

    #[test]
    fn trailing_coeng_binds_forward() {
        // ្hand at the end, so it stacks with what precedes without a base.
        assert_eq!(texts("ក្"), vec!["ក្"]);
    }

    #[test]
    fn coeng_stack_stays_together() {
        // ក ្ ត ្ រ: consecutive coeng+base pairs all bind into one cluster.
        assert_eq!(texts("ក្ត្រ"), vec!["ក្ត្រ"]);
    }

    #[test]
    fn leading_mark_is_own_cluster() {
        assert_eq!(texts("\u{17B6}a"), vec!["\u{17B6}", "a"]);
    }

    #[test]
    fn oov_diacritics_bind_to_base() {
        assert_eq!(texts("គ់"), vec!["គ់"]);
        assert_eq!(texts("គ៉ា"), vec!["គ៉ា"]);
        assert_eq!(texts("គ៍"), vec!["គ៍"]);
        assert_eq!(texts("ក័ង"), vec!["ក័", "ង"]);
    }

    #[test]
    fn inherent_vowels_and_extended_signs_bind_to_base() {
        for mark in [
            '\u{17B4}', '\u{17B5}', '\u{17CC}', '\u{17CE}', '\u{17CF}', '\u{17D1}', '\u{17D3}',
            '\u{17DD}',
        ] {
            let s = format!("ក{mark}");
            assert_eq!(texts(&s), vec![s.as_str()]);
        }
    }

    #[test]
    fn punctuation_and_currency_do_not_attach() {
        for sign in ['។', '៕', '៖', 'ៗ', '៘', '៙', '៚', '៛', 'ៜ'] {
            let s = format!("ក{sign}");
            assert_eq!(texts(&s), vec!["ក".to_owned(), sign.to_string()]);
        }
    }

    #[test]
    fn cluster_initial_coeng_binds_forward() {
        assert_eq!(texts("\u{17D2}ក"), vec!["\u{17D2}ក"]);
        assert_eq!(texts("\u{17D2}កា"), vec!["\u{17D2}កា"]);
        assert_eq!(texts("\u{17D2}ក\u{17D2}រា"), vec!["\u{17D2}ក\u{17D2}រា"]);
        assert_eq!(texts("\u{17D2}\u{17D2}ក"), vec!["\u{17D2}\u{17D2}ក"]);
        assert_eq!(texts("\u{17D2}កខ"), vec!["\u{17D2}ក", "ខ"]);
        assert_eq!(texts("\u{17D2}"), vec!["\u{17D2}"]);
    }

    #[test]
    fn empty_string() {
        assert_eq!(texts("").len(), 0);
    }
}
