//! Invisible-character cleanup, the first stage of the segmentation pipeline.
//!
//! [`normalize`] performs a deliberately *narrow* cleanup: it removes only
//! zero-width code points that render nothing and would poison both dictionary
//! keys and token offsets:
//!
//! - `U+200B` — zero-width space (ZWSP), used in web text as a soft break hint.
//! - `U+FEFF` — zero-width no-break space, appearing as a UTF-8 BOM leftover
//!   when encoded input is decoded naively.
//!
//! Why not full Unicode NFC: canonical composition/reordering would require
//! embedding complete decomposition tables (thousands of entries only available
//! behind external crates such as `unicode-normalization`, which this crate
//! forbids by design), for a benefit Khmer text barely shows — Khmer combining
//! marks (`U+17B6..U+17C8`) have no canonical composites to form except with
//! themselves already composed, and reordering out-of-canonical-order marks
//! changes nothing visible. The risk of over-engineering and subtly corrupting
//! offsets outweighs that. Downstream matching therefore works on the
//! *un-reordered* (but ZWSP/BOM-free) text; the dictionary in later tasks is
//! built from the same source corpus and inherits the same convention

use std::borrow::Cow;

/// Drops zero-width code points (`U+200B` ZWSP and `U+FEFF` BOM) from `s`.
///
/// The result is borrowed whenever `s` contains neither character, so the
/// common case costs nothing. Visible characters are never removed, so the
/// operation is lossless with respect to everything a user can see; see the
/// [module documentation](self) for why no full NFC pass is attempted.
///
/// Handles empty and non-Khmer input without special-casing and never panics.
///
/// # Examples
///
/// ```
/// use kh_lang_core::normalize;
///
/// assert_eq!(normalize("\u{200B}ខ្ញុំ\u{FEFF}"), "ខ្ញុំ");
/// ```
#[must_use]
pub fn normalize(s: &str) -> Cow<'_, str> {
    fn is_invisible(c: char) -> bool {
        c == '\u{200B}' || c == '\u{FEFF}'
    }
    if !s.chars().any(is_invisible) {
        return Cow::Borrowed(s);
    }
    let mut out = String::with_capacity(s.len());
    out.extend(s.chars().filter(|c| !is_invisible(*c)));
    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use super::normalize;
    use std::borrow::Cow;

    #[test]
    fn no_invisible_chars_returns_borrowed() {
        assert!(matches!(normalize("ខ្ញុំ abc 123"), Cow::Borrowed(_)));
    }

    #[test]
    fn removes_zwsp_and_bom() {
        assert_eq!(normalize("\u{200B}ខ្ញុំ\u{FEFF}"), "ខ្ញុំ");
    }

    #[test]
    fn keeps_all_visible_chars() {
        let input = "ខ្ញុំ ស្រលាញ់ abc 123 !@# ① ii \n  ";
        assert!(matches!(normalize(input), Cow::Borrowed(_)));
        assert_eq!(normalize(input), input);
    }

    #[test]
    fn empty_string() {
        let out = normalize("");
        assert_eq!(out, "");
        assert!(matches!(out, Cow::Borrowed(_)));
    }
}
