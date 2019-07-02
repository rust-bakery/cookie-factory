//! serialization library built with a combinator design similar to nom.
//!
//! Serializers are built up from single purpose serializers, like `slice`
//! to write a raw byte slice, or `be_u16` to write a `u16` integer in big
//! endian form.
//!
//! Those small serializers can then be assembled by using combinators.
//! As an example, `all(["abcd", "efgh", "ijkl"].iter().map(string))(output)`
//! will write `"abcdefghijkl"` to `output`.
#![cfg_attr(not(feature = "std"), no_std)]

/// lib module necessary to reexport what we need from std in `no_std` mode
pub mod lib {
  #[cfg(feature = "std")]
  pub mod std {
    pub mod io {
      pub use std::io::{Write, Result, Error};
    }
    pub use std::{cmp, fmt, iter, mem, result, slice};
  }

  #[cfg(not(feature = "std"))]
  pub mod std {
    pub use core::{cmp, iter, mem, result, slice};
    #[macro_use]
    pub use core::fmt;

    pub mod io {
      // there's no real io error on a byte slice
      pub type Error = ();
      pub type Result<T> = core::result::Result<T, Error>;

      pub trait Write {
        fn write(&mut self, buf: &[u8]) -> Result<usize>;
      }

      impl Write for &mut [u8] {
        fn write(&mut self, data: &[u8]) -> Result<usize> {
          let amt = core::cmp::min(data.len(), self.len());
          let (a, b) = core::mem::replace(self, &mut []).split_at_mut(amt);
          a.copy_from_slice(&data[..amt]);
          *self = b;
          Ok(amt)
        }
      }

    }
  }
}

pub use gen::*;
#[macro_use] mod gen;

mod combinators;
pub use combinators::*;

