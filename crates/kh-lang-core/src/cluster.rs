//! Grapheme-safe Khmer cluster iteration.
//!
//! Khmer stacks glyphs via the coeng ( subscribed-character) `U+17D2` placed
//! before a base consonant, followed by combining marks such as dependent
//! vowels `U+17B6..U+17C5`, nikhahit `U+17C6`, and the register shifts
//! `U+17C7..U+17C8`. A naive `str::chars()` iterator therefore slices through
//! the middle of a rendered glyph, breaking offset math in later matching and
//! slicing stages.
//!
//! [`clusters`] yields [`Cluster`]s with byte offsets into the input:
//!
//! - a coeng always binds to the character that follows it (their pair is kept
//!   together), transitively stacking consecutive `coeng + base` pairs;
//! - Khmer combining marks `U+17B6..U+17C8` attach to the cluster that
//!   precedes them (or form a lone cluster when nothing precedes);
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

/// Khmer combining marks: dependent vowels `U+17B6..U+17C5`, nikhahit
/// `U+17C6`, and the register-shift signs `U+17C7`/`U+17C8`.
fn is_khmer_mark(c: char) -> bool {
    ('\u{17B6}'..='\u{17C8}').contains(&c)
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
        if self.pos >= s.len() {
            return None;
        }
        let start = self.pos;
        let mut end = start + s[start..].chars().next()?.len_utf8();
        while let Some(c) = s[end..].chars().next() {
            if is_khmer_mark(c) {
                end += c.len_utf8();
            } else if c == COENG {
                // A coeng belongs to the base character that follows it; keep
                // the pair (and, transitively, a following coeng stack) inside
                // this cluster so it is never split from its strength char.
                end += c.len_utf8();
                if let Some(next) = s[end..].chars().next() {
                    end += next.len_utf8();
                }
            } else {
                break;
            }
        }
        self.pos = end;
        Some(Cluster {
            text: &s[start..end],
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
    fn empty_string() {
        assert_eq!(texts("").len(), 0);
    }
}
