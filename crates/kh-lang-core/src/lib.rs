//! Pure-Rust core of the Khmer text segmentation library.
//!
//! This crate is dependency-free and hosts the foundation layers that the
//! native Node.js bindings are built on:
//!
//! - [`normalizer`] — invisible-character cleanup ([`normalize`]) before any
//!   analysis.
//! - [`cluster`] — grapheme-safe Khmer cluster iteration ([`ClusterIter`],
//!   [`Cluster`]) so segmentation never splits inside stacked glyphs.
//! - [`run`] — script classification into `[Khmer][run::RunKind::Khmer] /
//!   [Other][run::RunKind::Other]` runs ([`runs`]) so algorithms only ever see
//!   homogeneous scripts.
//! - [`dictionary`] — the frequency-annotated word list ([`Dictionary`]),
//!   including the embedded SIL data.
//! - [`matcher`] — cluster-aligned matching over that dictionary with
//!   [`bimm`] (bidirectional maximal matching) and [`viterbi`] (score-based
//!   dynamic programming).
//! - [`segmenter`] — lossless word segmentation into [`Token`]s ([`Segmenter`],
//!   [`SegmenterBuilder`]) with a configurable [`Mode`] and dictionary, plus
//!   ZWSP insertion for display.
//!
//! The `napi-rs` glue lives at the repository root on top of this crate. The
//! core is pure: no `napi`, no `wasm`, no external dependencies.
//!
//! Guarantees:
//! - [`normalize`] is lossless with respect to *visible* characters: it only
//!   removes code points that render nothing (ZWSP, BOM). Concatenating
//!   cluster/`Run` byte-slices in order always reproduces the input exactly.
//! - Run, cluster and matcher offsets are UTF-8 byte offsets into the input,
//!   stable for slicing; [`Token`] offsets are UTF-16 code units for JS.

#![deny(missing_docs)]

pub mod cluster;
pub mod dictionary;
pub mod matcher;
pub mod normalizer;
pub mod run;
pub mod segmenter;

pub use cluster::{Cluster, ClusterIter, clusters};
pub use dictionary::Dictionary;
pub use matcher::{bimm, viterbi};
pub use normalizer::normalize;
pub use run::{Run, RunKind, runs};
pub use segmenter::{Mode, Segmenter, SegmenterBuilder, Token};
