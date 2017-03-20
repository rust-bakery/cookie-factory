//! Cookie factory
//! # cookie-factory
//!
//! serialization library built on the same design as nom.
//!
//! Highly experimental, don't use it if you're afraid of rewriting all your code
#![cfg_attr(feature = "nightly", feature(trace_macros))]

pub use gen::*;
#[macro_use] mod gen;
