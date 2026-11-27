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
use napi::bindgen_prelude::{
    Array, Either, FnArgs, Function, JsObjectValue, KeyCollectionMode, KeyConversion, KeyFilter,
    Object, This, Unknown, ValueType,
};
use napi::{Env, Error, JsGlobal, JsValue};
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
    env: Env,
    text: String,
    #[napi(ts_arg_type = "SegmentOptions | null | undefined")] options: Option<Unknown<'_>>,
) -> Result<Vec<String>> {
    let mode = parse_segment_options(&env, options)?;
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
        env: Env,
        #[napi(ts_arg_type = "SegmenterOptions | null | undefined")] options: Option<Unknown<'_>>,
    ) -> Result<Self> {
        let (mode, extras) = parse_segmenter_options(&env, options)?;
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

    /// Adds custom words in place and returns `this` (the same `Segmenter`
    /// instance) so calls can chain.
    ///
    /// Accepts an array of words (default frequency) or a plain object mapping
    /// words to frequencies. Duplicate words keep the last frequency. If
    /// validation fails, the receiver is left unchanged.
    #[napi(
        ts_args_type = "words: string[] | Record<string, number>",
        ts_return_type = "this"
    )]
    pub fn add_words<'env>(
        &mut self,
        env: Env,
        this: This<'env>,
        words: Unknown<'_>,
    ) -> Result<This<'env>> {
        let parsed = parse_words(&env, words, Status::Words, "words")?;
        // Validate every addition before touching the receiver.
        let additions = extras_from_words(parsed)?;
        self.extras.extend(additions);
        self.inner = Arc::new(build_segmenter(self.mode, &self.extras));
        Ok(this)
    }
}

/// Parses and validates the top-level [`SegmentOptions`].
fn parse_segment_options(env: &Env, options: Option<Unknown<'_>>) -> Result<kh_lang_core::Mode> {
    let Some(options) = options else {
        return Ok(kh_lang_core::Mode::default());
    };
    let object = expect_object(env, options, Status::Options, "options")?;
    Ok(parse_mode(&object)?
        .map(kh_lang_core::Mode::from)
        .unwrap_or_default())
}

/// Parses and validates the [`SegmenterOptions`] constructor argument.
fn parse_segmenter_options(
    env: &Env,
    options: Option<Unknown<'_>>,
) -> Result<(kh_lang_core::Mode, HashMap<String, u32>)> {
    let Some(options) = options else {
        return Ok((kh_lang_core::Mode::default(), HashMap::new()));
    };
    let object = expect_object(env, options, Status::Options, "options")?;
    let mode = parse_mode(&object)?
        .map(kh_lang_core::Mode::from)
        .unwrap_or_default();
    let extras = match parse_dictionary(env, &object)? {
        Some(words) => extras_from_words(words)?,
        None => HashMap::new(),
    };
    Ok((mode, extras))
}

/// Converts an [`Unknown`] into an [`Object`], rejecting non-objects.
fn expect_object<'env>(
    env: &Env,
    value: Unknown<'env>,
    status: Status,
    label: &str,
) -> Result<Object<'env>> {
    let value_type = value
        .get_type()
        .map_err(|error| Error::new(status, format!("invalid {label}: {error}")))?;
    if value_type != ValueType::Object {
        return Err(Error::new(
            status,
            format!("{label} must be an object, got {value_type}"),
        ));
    }
    // SAFETY: `value_type` is `Object`, so the cast is a plain wrapper around
    // the same napi handle; no JavaScript is invoked here.
    let object = unsafe { value.cast::<Object<'env>>() }
        .map_err(|error| Error::new(status, format!("invalid {label}: {error}")))?;
    let plain = !object.is_array().map_err(invalid(status, label))?
        && !is_branded(env, &object).map_err(invalid(status, label))?
        && has_object_prototype(env, &object).map_err(invalid(status, label))?;
    if !plain {
        return Err(Error::new(
            status,
            format!("{label} must be a plain object"),
        ));
    }
    Ok(object)
}

/// Maps a raw napi error onto `status`.
fn invalid(status: Status, label: &str) -> impl Fn(Error) -> Error<Status> + '_ {
    move |error| Error::new(status, format!("invalid {label}: {error}"))
}

/// Brands that `Object.getPrototypeOf` cannot reveal: these survive
/// `Object.setPrototypeOf(x, null)` and cross-realm transfer.
const UTIL_TYPES_BRANDS: [&str; 14] = [
    "isProxy",
    "isMap",
    "isSet",
    "isWeakMap",
    "isWeakSet",
    "isRegExp",
    "isBoxedPrimitive",
    "isSharedArrayBuffer",
    "isArgumentsObject",
    "isModuleNamespaceObject",
    "isGeneratorObject",
    "isExternal",
    "isMapIterator",
    "isSetIterator",
];

/// `(constructor, prototype member)` whose built-in getter or method throws
/// unless its receiver carries that constructor's internal slot. Used when
/// `util.types` is unavailable.
const SLOT_BRANDS: [(&str, &str); 11] = [
    ("Map", "size"),
    ("Set", "size"),
    ("WeakMap", "has"),
    ("WeakSet", "has"),
    ("RegExp", "dotAll"),
    ("Number", "valueOf"),
    ("String", "valueOf"),
    ("Boolean", "valueOf"),
    ("Symbol", "valueOf"),
    ("BigInt", "valueOf"),
    ("SharedArrayBuffer", "byteLength"),
];

/// Internal-slot brands `util.types` has no predicate for.
const EXTRA_SLOT_BRANDS: [(&str, &str); 2] =
    [("WeakRef", "deref"), ("FinalizationRegistry", "unregister")];

/// Whether `object` is a Proxy, a native-wrapped object (e.g. a `Segmenter`)
/// or a built-in exotic other than an array, judged from internal slots, so
/// it holds after `Object.setPrototypeOf` and across realms.
fn is_branded(env: &Env, object: &Object<'_>) -> napi::Result<bool> {
    if is_native_wrapped(env, object)
        || object.is_date()?
        || object.is_buffer()?
        || object.is_typedarray()?
        || object.is_arraybuffer()?
        || object.is_dataview()?
        || object.is_promise()?
        || object.is_error()?
    {
        return Ok(true);
    }
    let global = env.get_global()?;
    let fallback = match util_types(&global)? {
        Some(types) => {
            for brand in UTIL_TYPES_BRANDS {
                let check: Function<'_, Unknown<'_>, bool> = types.get_named_property(brand)?;
                if check.call(object.to_unknown())? {
                    return Ok(true);
                }
            }
            &[][..]
        }
        None => {
            // A Proxy reports a `null` prototype to N-API but its target's
            // prototype to `Reflect.getPrototypeOf`.
            let reflect: Object<'_> = global.get_named_property("Reflect")?;
            let get_prototype: Function<'_, Unknown<'_>, Unknown<'_>> =
                reflect.get_named_property("getPrototypeOf")?;
            if object.get_prototype()?.get_type()? == ValueType::Null
                && get_prototype.call(object.to_unknown())?.get_type()? != ValueType::Null
            {
                return Ok(true);
            }
            &SLOT_BRANDS[..]
        }
    };
    for (constructor, member) in fallback.iter().chain(&EXTRA_SLOT_BRANDS) {
        if has_slot(env, &global, constructor, member, object)? {
            return Ok(true);
        }
    }
    Ok(false)
}

/// Node's `util.types`, via `process.getBuiltinModule` (Node >= 20.16, 22.3).
fn util_types<'env>(global: &JsGlobal<'env>) -> napi::Result<Option<Object<'env>>> {
    let Some(process) = global.get_named_property::<Option<Object<'_>>>("process")? else {
        return Ok(None);
    };
    let Some(get_builtin) = process.get::<Function<'_, &str, Object<'_>>>("getBuiltinModule")?
    else {
        return Ok(None);
    };
    get_builtin
        .apply(process, "node:util")?
        .get_named_property("types")
        .map(Some)
}

/// Whether `napi_wrap` attached a native pointer to `object`.
fn is_native_wrapped(env: &Env, object: &Object<'_>) -> bool {
    let mut wrapped = std::ptr::null_mut();
    // SAFETY: `napi_unwrap` only reports whether a native pointer is attached
    // and never throws; `wrapped` is never dereferenced.
    let status = unsafe { napi::sys::napi_unwrap(env.raw(), object.raw(), &mut wrapped) };
    status == napi::sys::Status::napi_ok
}

/// Whether calling `globalThis[constructor].prototype[member]` (its getter for
/// accessors) on `object` succeeds, i.e. `object` has that internal slot.
///
/// A missing constructor counts as no slot. The expected `TypeError` is
/// cleared so it never leaks into later N-API calls.
fn has_slot(
    env: &Env,
    global: &JsGlobal<'_>,
    constructor: &str,
    member: &str,
    object: &Object<'_>,
) -> napi::Result<bool> {
    let constructor = global.get_named_property::<Unknown<'_>>(constructor)?;
    if constructor.get_type()? != ValueType::Function {
        return Ok(false);
    }
    let reflect: Object<'_> = global.get_named_property("Reflect")?;
    let get: Function<'_, FnArgs<(Unknown<'_>, &str)>, Unknown<'_>> =
        reflect.get_named_property("get")?;
    let describe: Function<'_, FnArgs<(Unknown<'_>, &str)>, Unknown<'_>> =
        reflect.get_named_property("getOwnPropertyDescriptor")?;
    let prototype = get.call((constructor, "prototype").into())?;
    let descriptor = describe.call((prototype, member).into())?;
    if descriptor.get_type()? != ValueType::Object {
        return Ok(false);
    }
    // SAFETY: `descriptor` has `ValueType::Object`; the cast only wraps the handle.
    let descriptor = unsafe { descriptor.cast::<Object<'_>>() }?;
    let function = if descriptor.has_own_property("get")? {
        descriptor.get_named_property::<Unknown<'_>>("get")?
    } else if descriptor.has_own_property("value")? {
        descriptor.get_named_property::<Unknown<'_>>("value")?
    } else {
        return Ok(false);
    };
    if function.get_type()? != ValueType::Function {
        return Ok(false);
    }
    // A fresh object satisfies the `WeakMap#has` / `unregister` argument.
    let argv = [Object::new(env)?.raw()];
    let mut result = std::ptr::null_mut();
    // SAFETY: every handle belongs to this `env` and outlives the call.
    let status = unsafe {
        napi::sys::napi_call_function(
            env.raw(),
            object.raw(),
            function.raw(),
            argv.len(),
            argv.as_ptr(),
            &mut result,
        )
    };
    match status {
        napi::sys::Status::napi_ok => Ok(true),
        napi::sys::Status::napi_pending_exception => {
            let mut exception = std::ptr::null_mut();
            // SAFETY: clears the brand check's own `TypeError`.
            napi::check_status!(unsafe {
                napi::sys::napi_get_and_clear_last_exception(env.raw(), &mut exception)
            })?;
            Ok(false)
        }
        status => napi::check_status!(status).map(|()| false),
    }
}

/// Whether `object`'s prototype is `null` or some realm's `Object.prototype`.
///
/// Cross-realm prototypes (e.g. from `node:vm`) are accepted only when their
/// own `constructor` is a native `Object` whose own `prototype` points back.
fn has_object_prototype(env: &Env, object: &Object<'_>) -> napi::Result<bool> {
    let proto = object.get_prototype()?;
    match proto.get_type()? {
        ValueType::Null => return Ok(true),
        ValueType::Object => {}
        _ => return Ok(false),
    }
    if env.strict_equals(proto, Object::new(env)?.get_prototype()?)? {
        return Ok(true);
    }
    // SAFETY: `proto` has `ValueType::Object`; the cast only wraps the handle.
    let proto_object = unsafe { proto.cast::<Object<'_>>() }?;
    if is_branded(env, &proto_object)?
        || proto_object.get_prototype()?.get_type()? != ValueType::Null
    {
        return Ok(false);
    }
    let global = env.get_global()?;
    let reflect: Object<'_> = global.get_named_property("Reflect")?;
    let describe: Function<'_, FnArgs<(Unknown<'_>, &str)>, Unknown<'_>> =
        reflect.get_named_property("getOwnPropertyDescriptor")?;
    let Some(ctor) = own_data_value(&describe, proto, "constructor")? else {
        return Ok(false);
    };
    if ctor.get_type()? != ValueType::Function {
        return Ok(false);
    }
    // SAFETY: `Function` and `Function.prototype` are functions, hence objects;
    // the casts only wrap the handles.
    let function_proto = unsafe {
        global
            .get_named_property::<Unknown<'_>>("Function")?
            .cast::<Object<'_>>()
    }?
    .get_named_property::<Unknown<'_>>("prototype")?;
    let to_string: Function<'_, (), String> =
        unsafe { function_proto.cast::<Object<'_>>() }?.get_named_property("toString")?;
    if to_string.apply(ctor, ())? != "function Object() { [native code] }" {
        return Ok(false);
    }
    match own_data_value(&describe, ctor, "prototype")? {
        Some(ctor_proto) => env.strict_equals(ctor_proto, proto),
        None => Ok(false),
    }
}

/// Reads `target[key]` only when it is an own data property, so no getter runs.
fn own_data_value<'env>(
    describe: &Function<'_, FnArgs<(Unknown<'_>, &str)>, Unknown<'env>>,
    target: Unknown<'_>,
    key: &str,
) -> napi::Result<Option<Unknown<'env>>> {
    let descriptor = describe.call((target, key).into())?;
    if descriptor.get_type()? != ValueType::Object {
        return Ok(None);
    }
    // SAFETY: `descriptor` has `ValueType::Object`; it is a fresh plain object.
    let descriptor = unsafe { descriptor.cast::<Object<'_>>() }?;
    if !descriptor.has_own_property("value")? {
        return Ok(None);
    }
    descriptor.get_named_property("value").map(Some)
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
fn parse_dictionary(env: &Env, object: &Object<'_>) -> Result<Option<ParsedWords>> {
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
        Some(dictionary) => parse_words(env, dictionary, Status::Options, "`dictionary`").map(Some),
    }
}

/// A JavaScript array of words or an object of word frequencies.
enum ParsedWords {
    /// Words without explicit frequencies.
    Names(Vec<String>),
    /// Words with explicit frequencies.
    Frequencies(Vec<(String, f64)>),
}

/// Parses a JavaScript array of strings or a plain object of numbers.
///
/// Arrays are distinguished from plain objects with a real array conversion
/// (an object cannot be read as a JavaScript array), so `[1, 2]` is never
/// mistaken for the frequency object `{"0": 1, "1": 2}`.
fn parse_words(env: &Env, value: Unknown<'_>, status: Status, label: &str) -> Result<ParsedWords> {
    let value_type = value
        .get_type()
        .map_err(|error| Error::new(status, format!("invalid {label}: {error}")))?;
    if value_type != ValueType::Object {
        return Err(Error::new(
            status,
            format!("{label} must be an array of strings or an object of word frequencies"),
        ));
    }

    let shape_error = || {
        Error::new(
            status,
            format!("{label} must be an array of strings or a plain object of word frequencies"),
        )
    };
    // SAFETY: the value is an object; the cast only wraps the handle.
    let object = unsafe { value.cast::<Object<'_>>() }.map_err(invalid(status, label))?;
    // Proxies are rejected before any trap can run, including `Proxy([])`.
    if is_branded(env, &object).map_err(invalid(status, label))? {
        return Err(shape_error());
    }

    if object.is_array().map_err(invalid(status, label))? {
        // SAFETY: `napi_is_array` confirmed a real JavaScript array.
        let array = unsafe { value.cast::<Array<'_>>() }.map_err(invalid(status, label))?;
        // Never preallocate from the JS-controlled length: a huge or holey
        // array must fail cheaply on its first missing entry instead.
        let mut words = Vec::new();
        for index in 0..array.len() {
            // Holes must not fall through to an inherited `Array.prototype[i]`.
            let entry_error =
                |error| Error::new(status, format!("invalid {label} entry {index}: {error}"));
            if !array
                .has_own_property(&index.to_string())
                .map_err(entry_error)?
            {
                return Err(Error::new(status, format!("missing {label} entry {index}")));
            }
            match array.get::<String>(index).map_err(entry_error)? {
                Some(word) => words.push(word),
                None => {
                    return Err(Error::new(status, format!("missing {label} entry {index}")));
                }
            }
        }
        return Ok(ParsedWords::Names(words));
    }

    if !has_object_prototype(env, &object).map_err(invalid(status, label))? {
        return Err(shape_error());
    }
    // Own enumerable keys only, read with the original JS key so lone
    // surrogates that collide after UTF-8 conversion are each validated.
    let keys = object
        .get_all_property_names(
            KeyCollectionMode::OwnOnly,
            KeyFilter::Enumerable,
            KeyConversion::NumbersToStrings,
        )
        .map_err(invalid(status, label))?;
    // SAFETY: `napi_get_all_property_names` always returns a JavaScript array.
    let keys = unsafe { keys.to_unknown().cast::<Array<'_>>() }.map_err(invalid(status, label))?;
    let mut frequencies = Vec::new();
    for index in 0..keys.len() {
        let Some(key) = keys
            .get::<Unknown<'_>>(index)
            .map_err(invalid(status, label))?
        else {
            continue;
        };
        // Symbol keys are ignored, matching `Object.keys`; a getter may also
        // have deleted a later key mid-iteration.
        if key.get_type().map_err(invalid(status, label))? != ValueType::String
            || !object
                .has_own_property_js(key)
                .map_err(invalid(status, label))?
        {
            continue;
        }
        let frequency = object.get_property::<_, f64>(key).map_err(|error| {
            Error::new(status, format!("invalid frequency in {label}: {error}"))
        })?;
        let name = key
            .coerce_to_string()
            .and_then(|name| name.into_utf8())
            .and_then(|name| name.into_owned())
            .map_err(invalid(status, label))?;
        frequencies.push((name, frequency));
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
