//! serialization library built with a combinator design similar to nom.
//!
//! Serializers are built up from single purpose serializers, like `slice`
//! to write a raw byte slice, or `be_u16` to write a `u16` integer in big
//! endian form.
//!
//! Those small serializers can then be assembled by using combinators.
//! As an example, `all(["abcd", "efgh", "ijkl"].iter().map(string))(output)`
//! will write `"abcdefghijkl"` to `output`.
//!
#![cfg_attr(feature = "nightly", feature(trace_macros))]

pub use gen::*;
#[macro_use] mod gen;

mod combinators;
pub use combinators::*;

mod cftrait;
pub use cftrait::*;
