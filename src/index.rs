//! Node.js bindings for the pure-Rust [`kh_lang_core`] crate.
//!
//! This crate is deliberately translation-only: it maps napi values onto the
//! core API, validates JavaScript input and surfaces stable error codes. All
//! segmentation logic lives in `kh-lang-core`.
//!
//! # JavaScript surface
//!
//! - [`segment`] — top-level convenience function returning token texts.
//! - [`normalize`] — invisible-character cleanup.
//! - [`Segmenter`] — reusable segmenter with custom words and token offsets.
//! - [`Token`], [`SegmentOptions`], [`SegmenterOptions`] — TypeScript shapes.

use std::collections::HashMap;
use std::sync::Arc;

use kh_lang_core::dictionary::DEFAULT_FREQ;
use kh_lang_core::{Segmenter as CoreSegmenter, Token as CoreToken};
use napi::Error;
use napi::bindgen_prelude::{Array, Either, JsObjectValue, Object, Unknown, ValueType};
use napi_derive::napi;

/// Stable error status codes exposed as `error.code` in JavaScript.
///
/// The variants are intentionally named after their JavaScript codes so a
/// thrown error is self-describing in both languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// Invalid or unknown options, or a malformed options object.
    Options,
    /// An invalid custom word or frequency.
    Words,
}

impl AsRef<str> for Status {
    fn as_ref(&self) -> &str {
        match self {
            Status::Options => "ERR_KH_OPTIONS",
            Status::Words => "ERR_KH_WORDS",
        }
    }
}

/// Result type carrying a stable [`Status`] error code.
pub type Result<T> = std::result::Result<T, Error<Status>>;

/// Builds an [`ERR_KH_OPTIONS`](Status::Options) error.
fn options_error(message: impl Into<String>) -> Error<Status> {
    Error::new(Status::Options, message.into())
}

/// Builds an [`ERR_KH_WORDS`](Status::Words) error.
fn words_error(message: impl Into<String>) -> Error<Status> {
    Error::new(Status::Words, message.into())
}

/// Segmentation algorithm.
///
/// The generated TypeScript type is `'bimm' | 'viterbi'`; the default is
/// `'bimm'` everywhere.
#[napi(string_enum = "lowercase")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SegmentMode {
    /// Bidirectional maximal matching (default).
    #[default]
    Bimm,
    /// Frequency-scored Viterbi dynamic programming.
    Viterbi,
}

impl From<SegmentMode> for kh_lang_core::Mode {
    fn from(mode: SegmentMode) -> Self {
        match mode {
            SegmentMode::Bimm => Self::Bimm,
            SegmentMode::Viterbi => Self::Viterbi,
        }
    }
}

/// Options accepted by the top-level [`segment`] function.
#[napi(object)]
#[derive(Debug, Clone, Default)]
pub struct SegmentOptions {
    /// Segmentation algorithm; defaults to `'bimm'`.
    pub mode: Option<SegmentMode>,
}

/// Options accepted by the [`Segmenter`] constructor.
#[napi(object)]
#[derive(Debug, Clone, Default)]
pub struct SegmenterOptions {
    /// Segmentation algorithm; defaults to `'bimm'`.
    pub mode: Option<SegmentMode>,
    /// Additional words layered on top of the embedded dictionary.
    ///
    /// An array of words uses the core default frequency; an object maps each
    /// word to its frequency. The embedded dictionary is never replaced.
    pub dictionary: Option<Either<Vec<String>, HashMap<String, f64>>>,
}

/// One segment of the original input.
#[napi(object)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    /// The token's exact characters from the input.
    pub text: String,
    /// UTF-16 code-unit offset of the token's first code unit in the input.
    pub start: u32,
    /// UTF-16 code-unit offset one past the token's last code unit.
    pub end: u32,
}

impl From<CoreToken> for Token {
    fn from(token: CoreToken) -> Self {
        Self {
            text: token.text,
            start: token.start,
            end: token.end,
        }
    }
}

/// Segments `text` into word tokens using the embedded dictionary.
///
/// Returns the token texts only; use [`Segmenter::segment`] when UTF-16
/// offsets are needed. The default mode is `'bimm'`, and concatenating the
/// returned strings always reproduces `text` exactly.
#[napi]
pub fn segment(
    text: String,
    #[napi(ts_arg_type = "SegmentOptions | null | undefined")] options: Option<Unknown<'_>>,
) -> Result<Vec<String>> {
    let mode = parse_segment_options(options)?;
    let segmenter = CoreSegmenter::builder().mode(mode).build();
    Ok(segmenter
        .segment(&text)
        .into_iter()
        .map(|token| token.text)
        .collect())
}

/// Removes `U+200B` ZWSP and `U+FEFF` BOM only.
///
/// All other characters, including Khmer combining marks, are preserved
/// unchanged and never reordered.
#[napi]
pub fn normalize(text: String) -> String {
    kh_lang_core::normalize(&text).into_owned()
}

/// A reusable Khmer word segmenter with an optional mode and custom words.
///
/// Custom words are additive: they layer on top of the embedded dictionary
/// rather than replacing it.
#[napi]
#[derive(Clone)]
pub struct Segmenter {
    inner: Arc<CoreSegmenter>,
    mode: kh_lang_core::Mode,
    extras: HashMap<String, u32>,
}

#[napi]
impl Segmenter {
    /// Creates a segmenter backed by the embedded dictionary.
    ///
    /// `mode` defaults to `'bimm'`. `dictionary` adds custom words; duplicate
    /// words keep the last frequency supplied.
    #[napi(constructor)]
    pub fn new(
        #[napi(ts_arg_type = "SegmenterOptions | null | undefined")] options: Option<Unknown<'_>>,
    ) -> Result<Self> {
        let (mode, extras) = parse_segmenter_options(options)?;
        let inner = Arc::new(build_segmenter(mode, &extras));
        Ok(Self {
            inner,
            mode,
            extras,
        })
    }

    /// Segments `text` into contiguous, lossless [`Token`]s with UTF-16
    /// offsets.
    #[napi]
    pub fn segment(&self, text: String) -> Vec<Token> {
        self.inner
            .segment(&text)
            .into_iter()
            .map(Token::from)
            .collect()
    }

    /// Alias for [`Segmenter::segment`].
    ///
    /// Both methods return identical `Token[]` values; `split` exists for API
    /// compatibility.
    #[napi]
    pub fn split(&self, text: String) -> Vec<Token> {
        self.inner
            .segment(&text)
            .into_iter()
            .map(Token::from)
            .collect()
    }

    /// Segments `text` and inserts U+200B ZWSP between adjacent Khmer words.
    ///
    /// Removing the inserted ZWSP restores the input; pre-existing ZWSP are
    /// preserved.
    #[napi]
    pub fn insert_zwsp(&self, text: String) -> String {
        self.inner.insert_zwsp(&text)
    }

    /// Adds custom words and returns this segmenter so calls can chain.
    ///
    /// Accepts an array of words (default frequency) or an object mapping
    /// words to frequencies. Duplicate words keep the last frequency. The
    /// receiver is mutated and a cheap copy is returned, so
    /// `new Segmenter().addWords([...]).segment(...)` works.
    #[napi]
    pub fn add_words(
        &mut self,
        #[napi(ts_arg_type = "string[] | Record<string, number>")] words: Unknown<'_>,
    ) -> Result<Self> {
        let parsed = parse_words(words, Status::Words, "words")?;
        // Validate every addition before touching the receiver.
        let additions = extras_from_words(parsed)?;
        self.extras.extend(additions);
        self.inner = Arc::new(build_segmenter(self.mode, &self.extras));
        Ok(self.clone())
    }
}

/// Parses and validates the top-level [`SegmentOptions`].
fn parse_segment_options(options: Option<Unknown<'_>>) -> Result<kh_lang_core::Mode> {
    let Some(options) = options else {
        return Ok(kh_lang_core::Mode::default());
    };
    let object = expect_object(options, Status::Options, "options")?;
    Ok(parse_mode(&object)?
        .map(kh_lang_core::Mode::from)
        .unwrap_or_default())
}

/// Parses and validates the [`SegmenterOptions`] constructor argument.
fn parse_segmenter_options(
    options: Option<Unknown<'_>>,
) -> Result<(kh_lang_core::Mode, HashMap<String, u32>)> {
    let Some(options) = options else {
        return Ok((kh_lang_core::Mode::default(), HashMap::new()));
    };
    let object = expect_object(options, Status::Options, "options")?;
    let mode = parse_mode(&object)?
        .map(kh_lang_core::Mode::from)
        .unwrap_or_default();
    let extras = match parse_dictionary(&object)? {
        Some(words) => extras_from_words(words)?,
        None => HashMap::new(),
    };
    Ok((mode, extras))
}

/// Converts an [`Unknown`] into an [`Object`], rejecting non-objects.
fn expect_object<'env>(value: Unknown<'env>, status: Status, label: &str) -> Result<Object<'env>> {
    let value_type = value
        .get_type()
        .map_err(|error| Error::new(status, format!("invalid {label}: {error}")))?;
    if value_type != ValueType::Object {
        return Err(Error::new(
            status,
            format!("{label} must be an object, got {value_type}"),
        ));
    }
    // SAFETY: the value is an object. `Array` conversion succeeds only for a
    // real JavaScript array; options must be a plain object.
    if unsafe { value.cast::<Array<'_>>() }.is_ok() {
        return Err(Error::new(
            status,
            format!("{label} must be an object, not an array"),
        ));
    }
    // SAFETY: `value_type` is `Object`, so the cast is a plain wrapper around
    // the same napi handle; no JavaScript is invoked here.
    unsafe { value.cast::<Object<'env>>() }
        .map_err(|error| Error::new(status, format!("invalid {label}: {error}")))
}

/// Reads the optional `mode` property from an options object.
fn parse_mode(object: &Object<'_>) -> Result<Option<SegmentMode>> {
    // Only own properties count: an inherited `Object.prototype.mode` must not
    // inject an option.
    if !object
        .has_own_property("mode")
        .map_err(|error| options_error(format!("invalid `mode` option: {error}")))?
    {
        return Ok(None);
    }
    let mode = object
        .get::<Option<String>>("mode")
        .map_err(|error| options_error(format!("invalid `mode` option: {error}")))?
        .flatten();
    match mode {
        None => Ok(None),
        Some(mode) => parse_mode_value(&mode).map(Some),
    }
}

/// Converts the JavaScript mode string into a [`SegmentMode`].
fn parse_mode_value(mode: &str) -> Result<SegmentMode> {
    match mode {
        "bimm" => Ok(SegmentMode::Bimm),
        "viterbi" => Ok(SegmentMode::Viterbi),
        _ => Err(options_error(format!(
            "unknown mode `{mode}`; expected `bimm` or `viterbi`"
        ))),
    }
}

/// Reads the optional `dictionary` property from an options object.
fn parse_dictionary(object: &Object<'_>) -> Result<Option<ParsedWords>> {
    // Only own properties count: an inherited `Object.prototype.dictionary`
    // must not inject an option.
    if !object
        .has_own_property("dictionary")
        .map_err(|error| options_error(format!("invalid `dictionary` option: {error}")))?
    {
        return Ok(None);
    }
    let dictionary = object
        .get::<Option<Unknown<'_>>>("dictionary")
        .map_err(|error| options_error(format!("invalid `dictionary` option: {error}")))?
        .flatten();
    match dictionary {
        None => Ok(None),
        Some(dictionary) => parse_words(dictionary, Status::Options, "`dictionary`").map(Some),
    }
}

/// A JavaScript array of words or an object of word frequencies.
enum ParsedWords {
    /// Words without explicit frequencies.
    Names(Vec<String>),
    /// Words with explicit frequencies.
    Frequencies(HashMap<String, f64>),
}

/// Parses a JavaScript array of strings or a plain object of numbers.
///
/// Arrays are distinguished from plain objects with a real array conversion
/// (an object cannot be read as a JavaScript array), so `[1, 2]` is never
/// mistaken for the frequency object `{"0": 1, "1": 2}`.
fn parse_words(value: Unknown<'_>, status: Status, label: &str) -> Result<ParsedWords> {
    let value_type = value
        .get_type()
        .map_err(|error| Error::new(status, format!("invalid {label}: {error}")))?;
    if value_type != ValueType::Object {
        return Err(Error::new(
            status,
            format!("{label} must be an array of strings or an object of word frequencies"),
        ));
    }

    // SAFETY: the value is an object. `Array` conversion fails for plain
    // objects, which is how the two accepted shapes are told apart.
    if let Ok(array) = unsafe { value.cast::<Array<'_>>() } {
        // Never preallocate from the JS-controlled length: a huge or holey
        // array must fail cheaply on its first missing entry instead.
        let mut words = Vec::new();
        for index in 0..array.len() {
            let word = array.get::<String>(index).map_err(|error| {
                Error::new(status, format!("invalid {label} entry {index}: {error}"))
            })?;
            match word {
                Some(word) => words.push(word),
                None => {
                    return Err(Error::new(status, format!("missing {label} entry {index}")));
                }
            }
        }
        return Ok(ParsedWords::Names(words));
    }

    // SAFETY: the value is an object, as checked above.
    let object = unsafe { value.cast::<Object<'_>>() }
        .map_err(|error| Error::new(status, format!("invalid {label}: {error}")))?;
    let keys = Object::keys(&object)
        .map_err(|error| Error::new(status, format!("invalid {label}: {error}")))?;
    let mut frequencies = HashMap::with_capacity(keys.len());
    for key in keys {
        let frequency = object
            .get::<f64>(&key)
            .map_err(|error| Error::new(status, format!("invalid frequency for `{key}`: {error}")))?
            .ok_or_else(|| Error::new(status, format!("missing frequency for `{key}`")))?;
        frequencies.insert(key, frequency);
    }
    Ok(ParsedWords::Frequencies(frequencies))
}

/// Validates parsed words and turns them into a word → frequency map.
fn extras_from_words(words: ParsedWords) -> Result<HashMap<String, u32>> {
    let mut extras = HashMap::new();
    match words {
        ParsedWords::Names(names) => {
            for name in names {
                extras.insert(validate_word(name)?, DEFAULT_FREQ);
            }
        }
        ParsedWords::Frequencies(frequencies) => {
            for (name, frequency) in frequencies {
                extras.insert(validate_word(name)?, validate_frequency(frequency)?);
            }
        }
    }
    Ok(extras)
}

/// Rejects empty, whitespace-only, or line-breaking words.
fn validate_word(word: String) -> Result<String> {
    if word.trim().is_empty() {
        return Err(words_error("word must not be empty or whitespace-only"));
    }
    if word
        .chars()
        .any(|character| matches!(character, '\t' | '\r' | '\n'))
    {
        return Err(words_error(
            "word must not contain tab, carriage return, or newline",
        ));
    }
    Ok(word)
}

/// Validates a frequency and truncates fractions toward zero.
fn validate_frequency(frequency: f64) -> Result<u32> {
    if !frequency.is_finite() || frequency < 1.0 || frequency > f64::from(u32::MAX) {
        return Err(words_error(
            "frequency must be a finite number between 1 and 4294967295",
        ));
    }
    Ok(frequency.trunc() as u32)
}

/// Rebuilds the core segmenter from the embedded dictionary plus `extras`.
fn build_segmenter(mode: kh_lang_core::Mode, extras: &HashMap<String, u32>) -> CoreSegmenter {
    if extras.is_empty() {
        return CoreSegmenter::builder().mode(mode).build();
    }
    let mut dictionary = kh_lang_core::Dictionary::embedded().clone();
    for (word, frequency) in extras {
        dictionary.insert(word, *frequency);
    }
    CoreSegmenter::builder()
        .mode(mode)
        .dictionary(dictionary)
        .build()
}
