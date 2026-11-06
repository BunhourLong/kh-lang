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
//!
//! Later tasks layer dictionary matching (`dictionary`, `matcher`),
//! segmentation (`Segmenter`) and the `napi-rs` glue on top of these. The
//! crate is pure: no `napi`, no `wasm`, no external dependencies.
//!
//! Guarantees relevant to later tasks:
//! - [`normalize`] is lossless with respect to *visible* characters: it only
//!   removes code points that render nothing (ZWSP, BOM). Concatenating
//!   cluster/`Run` byte-slices in order always reproduces the input exactly.
//! - Offsets are byte offsets into the input string, stable for slicing.

#![deny(missing_docs)]

pub mod cluster;
pub mod normalizer;
pub mod run;

pub use cluster::{Cluster, ClusterIter, clusters};
pub use normalizer::normalize;
pub use run::{Run, RunKind, runs};
