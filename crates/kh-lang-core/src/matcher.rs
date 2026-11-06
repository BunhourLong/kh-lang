//! Dictionary matchers producing cluster-aligned token byte ranges.
//!
//! Both matchers take any string and return [`Range`]s of its **UTF-8 byte
//! offsets**, aligned to the grapheme-safe cluster boundaries of
//! [`crate::cluster`] — clustering is done once and clusters are the atomic
//! units of every match, so no token can ever split a stacked Khmer glyph.
//! Concatenating `text[r.start..r.end]` over the returned ranges (in order)
//! reproduces the input exactly; empty input yields no ranges.
//!
//! - [`bimm`]: bidirectional incremental/ maximal matching — a forward
//!   greedy pass and a backward greedy pass, then the path with fewer tokens
//!   wins; ties prefer the forward path (documented decision D4).
//! - [`viterbi`]: unigram HMM-lite dynamic program over the dictionary,
//!   `score(word) = ln(freq / Σfreq)` (f64 `ln`, with `freq <= 0` treated as
//!   `1` before taking the log so the log is finite). Candidates the
//!   dictionary does not cover get a small constant emission cost; because a
//!   single-cluster fallback edge always exists, every OOV character is
//!   *emitted* (never dropped) and OOV stretches come out as one cluster per
//!   token.
//!
//! Caller note: these matchers treat the whole string uniformly (the
//! segmenter task layers run classification on top); the public API contract
//! for this crate keeps them available for arbitrary input.

use std::ops::Range;

use crate::clusters;
use crate::dictionary::Dictionary;

/// Bidirectional incremental/maximal matching.
///
/// Runs a forward greedy longest-match pass and a backward greedy
/// longest-match pass, then keeps the path with fewer tokens; ties prefer
/// the forward path. Dictionary misses fall back to one cluster per token.
///
/// # Examples
///
/// ```
/// use kh_lang_core::dictionary::Dictionary;
/// use kh_lang_core::matcher::bimm;
///
/// let ranges = bimm(Dictionary::embedded(), "ខ្ញុំស្រលាញ់ភ្នំពេញ");
/// assert_eq!(ranges.len(), 3);
/// ```
#[must_use]
pub fn bimm(dict: &Dictionary, text: &str) -> Vec<Range<usize>> {
    let cl = Bounds::collect(text);
    let forward = forward_greedy(dict, text, &cl);
    let backward = backward_greedy(dict, text, &cl);
    if backward.len() < forward.len() {
        backward
    } else {
        forward
    }
}

/// Unigram dynamic-programming matcher ("HMM-lite").
///
/// Every dictionary word spanning whole clusters scores
/// `ln(freq / Σfreq)`; an unseen (OOV) span gets the constant cost
/// `ln(1 / Σfreq) - 5.0` (the documented `OOV_PENALTY` constant). Because
/// `Σfreq >= 1` and the penalty is positive, any real dictionary word
/// (`freq >= 1`) outscores an OOV token spanning the same clusters. The DP
/// maximizes the total score and breaks ties toward the longer first token, so
/// results are deterministic; OOV text still splits into one token per cluster
/// rather than ever discarding characters.
#[must_use]
pub fn viterbi(dict: &Dictionary, text: &str) -> Vec<Range<usize>> {
    let cl = Bounds::collect(text);
    viterbi_dp(dict, text, &cl)
}

/// Penalty subtracted from the baseline OOV score `ln(1 / Σfreq)`.
///
/// The constant only has to be strictly positive so that every dictionary
/// word (whose score is at worst `ln(1 / Σfreq)` for `freq = 1`) beats an OOV
/// token covering the same clusters. `5.0` additionally keeps OOV stretches
/// from being merged speculatively: splitting them one cluster at a time wins
/// by a wide margin.
const OOV_PENALTY: f64 = 5.0;

/// Cluster boundary table: byte `starts[i]..ends[i]` for cluster `i`.
struct Bounds {
    starts: Vec<usize>,
    ends: Vec<usize>,
}

impl Bounds {
    fn collect(text: &str) -> Self {
        let mut starts = Vec::new();
        let mut ends = Vec::new();
        for cluster in clusters(text) {
            starts.push(cluster.byte_start);
            ends.push(cluster.byte_end);
        }
        Self { starts, ends }
    }
}

/// Forward greedy longest match: at each cluster, keep going while the
/// accumulated span can still be a dictionary word; commit the longest hit,
/// else the single cluster.
fn forward_greedy(dict: &Dictionary, text: &str, cl: &Bounds) -> Vec<Range<usize>> {
    let n = cl.starts.len();
    let max = dict.max_word_len();
    let mut out = Vec::with_capacity(n);
    let mut k = 0;
    while k < n {
        let start = cl.starts[k];
        let mut best: Option<(usize, usize)> = None; // (byte end, cluster index)
        let mut j = k;
        loop {
            let end = cl.ends[j];
            if end - start > max {
                break;
            }
            if dict.contains(&text[start..end]) {
                best = Some((end, j));
            }
            if j + 1 >= n {
                break;
            }
            j += 1;
        }
        match best {
            Some((end, next)) => {
                out.push(start..end);
                k = next + 1;
            }
            None => {
                out.push(start..cl.ends[k]);
                k += 1;
            }
        }
    }
    out
}

/// Backward greedy longest match: walk clusters right-to-left committing the
/// longest dictionary word ending at each boundary, else the single cluster
/// (output is reversed back into source order).
fn backward_greedy(dict: &Dictionary, text: &str, cl: &Bounds) -> Vec<Range<usize>> {
    let n = cl.starts.len();
    let max = dict.max_word_len();
    let mut out = Vec::with_capacity(n);
    let mut j = n;
    while j > 0 {
        let end = cl.ends[j - 1];
        let mut best: Option<(usize, usize)> = None; // (byte start, cluster index)
        let mut i = j - 1;
        loop {
            let start = cl.starts[i];
            if end - start > max {
                break;
            }
            if dict.contains(&text[start..end]) {
                best = Some((start, i));
            }
            if i == 0 {
                break;
            }
            i -= 1;
        }
        match best {
            Some((start, prev)) => {
                out.push(start..end);
                j = prev;
            }
            None => {
                out.push(cl.starts[j - 1]..end);
                j -= 1;
            }
        }
    }
    out.reverse();
    out
}

/// Viterbi DP over cluster boundaries, computed right-to-left so ties can
/// prefer the longer *first* token.
///
/// A candidate edge `k..j` scores `ln(freq / Σfreq) + best[j]`; an OOV
/// single-cluster edge scores `ln(1 / Σfreq) - OOV_PENALTY + best[j]`. The
/// best edge is kept, with strict `>` comparison while candidates are visited
/// longest-first, which resolves equal scores deterministically toward the
/// longer token starting at `k`.
fn viterbi_dp(dict: &Dictionary, text: &str, cl: &Bounds) -> Vec<Range<usize>> {
    let n = cl.starts.len();
    let total = dict.total_freq().max(1.0);
    let log_total = total.ln();
    let oov_score = -log_total - OOV_PENALTY;
    let max = dict.max_word_len();

    // best[k]: highest score covering clusters k..n; next[k]: its first edge end.
    let mut best = vec![f64::NEG_INFINITY; n + 1];
    let mut next = vec![0usize; n + 1];
    best[n] = 0.0;

    for k in (0..n).rev() {
        // OOV fallback: emit exactly cluster k.
        let mut best_score = oov_score + best[k + 1];
        let mut best_j = k + 1;

        // Largest end index reachable without exceeding the longest word.
        let mut max_j = k + 1;
        while max_j < n && cl.ends[max_j] - cl.starts[k] <= max {
            max_j += 1;
        }

        // Longest-first, strict improvement: equal scores keep the longer token.
        for j in (k + 1..=max_j).rev() {
            if let Some(freq) = dict.freq(&text[cl.starts[k]..cl.ends[j - 1]]) {
                let freq = f64::from(freq.max(1));
                let score = (freq.ln() - log_total) + best[j];
                if score > best_score {
                    best_score = score;
                    best_j = j;
                }
            }
        }

        best[k] = best_score;
        next[k] = best_j;
    }

    // Backtrack; every position has a finite OOV chain, so `next` is always set.
    let mut out = Vec::with_capacity(n);
    let mut k = 0;
    while k < n {
        let j = if next[k] > k { next[k] } else { k + 1 };
        out.push(cl.starts[k]..cl.ends[j - 1]);
        k = j;
    }
    out
}

#[cfg(test)]
mod tests {
    use std::ops::Range;

    use super::{bimm, viterbi};
    use crate::cluster::clusters;
    use crate::dictionary::Dictionary;

    fn spans<'a>(text: &'a str, ranges: &[Range<usize>]) -> Vec<&'a str> {
        ranges.iter().map(|r| &text[r.clone()]).collect()
    }

    /// Asserts the ranges are non-empty, contiguous, cover the whole input,
    /// and that slicing them reproduces `text` exactly.
    fn assert_lossless(text: &str, ranges: &[Range<usize>]) {
        let mut cursor = 0;
        for r in ranges {
            assert_eq!(r.start, cursor, "ranges must be contiguous: {ranges:?}");
            assert!(r.start < r.end, "empty range in {ranges:?}");
            assert!(r.end <= text.len(), "range past input end in {ranges:?}");
            cursor = r.end;
        }
        assert_eq!(cursor, text.len(), "ranges must cover the whole input");
        let flat: String = ranges.iter().map(|r| &text[r.clone()]).collect();
        assert_eq!(flat, text, "ranges must reproduce the input");
    }

    #[test]
    fn khanhont_sentence_is_two_to_four_tokens_in_both_modes() {
        let text = "ខ្ញុំស្រលាញ់ភ្នំពេញ";
        let dict = Dictionary::embedded();
        for ranges in [bimm(dict, text), viterbi(dict, text)] {
            assert!(
                (2..=4).contains(&ranges.len()),
                "expected 2-4 tokens, got {} ({:?})",
                ranges.len(),
                spans(text, &ranges)
            );
            assert_eq!(spans(text, &ranges), vec!["ខ្ញុំ", "ស្រលាញ់", "ភ្នំពេញ"]);
            assert_lossless(text, &ranges);
        }
    }

    #[test]
    fn oov_string_is_lossless_under_both_matchers() {
        // No multi-cluster dictionary word covers this string, so both
        // matchers must fall back to cluster-sized tokens without dropping
        // anything.
        let text = "គគគគគ";
        let dict = Dictionary::embedded();
        let cluster_count = clusters(text).count();
        for ranges in [bimm(dict, text), viterbi(dict, text)] {
            assert_eq!(ranges.len(), cluster_count);
            assert_lossless(text, &ranges);
        }
    }

    #[test]
    fn mixed_scripts_emoji_and_trailing_coeng_are_lossless() {
        let dict = Dictionary::embedded();
        // Digits, ZWSP, emoji, a full Khmer word and a dangling coeng.
        let text = "a\u{200B}ខ្ញុំ🙂123ក្";
        for ranges in [bimm(dict, text), viterbi(dict, text)] {
            assert_lossless(text, &ranges);
        }
    }

    #[test]
    fn welcome_sentence_both_matchers_agree_on_dictionary_compounds() {
        // All three outputs are dictionary words: សូមស្វាគមន៍ (46),
        // មកកាន់ (69), ប្រទេសកម្ពុជា (577). Three tokens is the correct
        // segmentation here — do not force a higher count.
        let text = "សូមស្វាគមន៍មកកាន់ប្រទេសកម្ពុជា";
        let dict = Dictionary::embedded();
        let viterbi_ranges = viterbi(dict, text);
        let bimm_ranges = bimm(dict, text);
        let expected = ["សូមស្វាគមន៍", "មកកាន់", "ប្រទេសកម្ពុជា"];
        assert_eq!(spans(text, &viterbi_ranges), expected);
        assert_eq!(bimm_ranges, viterbi_ranges);
        assert_lossless(text, &viterbi_ranges);
    }

    #[test]
    fn both_matchers_are_deterministic() {
        let dict = Dictionary::embedded();
        for text in ["ខ្ញុំស្រលាញ់ភ្នំពេញ", "សូមស្វាគមន៍មកកាន់ប្រទេសកម្ពុជា", "គគគគគ", "abc ខ្ញុំ 123"]
        {
            assert_eq!(bimm(dict, text), bimm(dict, text));
            assert_eq!(viterbi(dict, text), viterbi(dict, text));
        }
    }

    #[test]
    fn empty_input_yields_no_ranges() {
        let dict = Dictionary::embedded();
        assert!(bimm(dict, "").is_empty());
        assert!(viterbi(dict, "").is_empty());
    }

    #[test]
    fn ranges_align_to_cluster_boundaries() {
        // Every range must coincide with a cluster offset pair.
        let text = "abcខ្ញុំស្រលាញ់ភ្នំពេញx";
        let dict = Dictionary::embedded();
        let edges: Vec<usize> = clusters(text)
            .flat_map(|c| [c.byte_start, c.byte_end])
            .collect();
        for mode in [bimm(dict, text), viterbi(dict, text)] {
            assert_lossless(text, &mode);
            for r in mode {
                assert!(r.start < r.end);
                assert!(
                    edges.contains(&r.start),
                    "start {} not a cluster edge",
                    r.start
                );
                assert!(edges.contains(&r.end), "end {} not a cluster edge", r.end);
            }
        }
    }
}
