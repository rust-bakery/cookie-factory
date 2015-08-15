//! Cookie factory
#![feature(trace_macros)]
//macros:
//* write_vec!(vec, ...) -> Result(nb of bytes)
//* write_slice!(s, ...) -> Result(nb of bytes)
//* write_iterator!(it, ...) -> Result(nb of bytes)
//
//* config can be endianness
// combinator: c!(writer, config, conbinator/serializer)

/// doc
///
enum WriteMode {
  Slice,
  Byte
}

#[macro_export]
macro_rules! write_vec (
  ($vec:expr, $config:expr, Slice, $value: expr) => (
    ($vec).extend(($value).iter().cloned());
  );
  ($vec:expr, $config:expr, Byte, $value: expr) => (
    ($vec).push($value);
  );
);

/*
#[macro_export]
macro_rules! write_slice (
  ($sl:expr, $($wargs:tt)*) => (
    let mut pos = 0;
    write_slice_impl!(($sl, pos), $($wargs)*)
  );
);
*/

#[macro_export]
macro_rules! write_slice (
  (($sl:expr, $pos:expr), $config:expr, Slice, $value: expr) => (
    {
      std::slice::bytes::copy_memory($value, &mut ($sl)[$pos..($value.length)]);
      $pos += value.length;
    }
  );
  (($sl:expr, $pos:expr), $config:expr, Byte, $value: expr) => (
    {
      $sl[$pos] = $value as u8;
      $pos += 1;
    }
  );
);

#[macro_export]
macro_rules! opt (
  ($writer:ident!($($wargs:tt)*), $config:expr, $submac:ident!( $($args:tt)* )) => (
    {
      match $submac!($writer!($($wargs)*), $config, $($args)*) {
        Ok(nb) => Ok(nb),
        Err(_) => Ok(0)
      }
    }
  )
);

#[macro_export]
macro_rules! s_u8 (
  ($writer:ident!($($wargs:tt)*), $config:expr, $value:expr) => (
    {
      $writer!($($wargs)*, $config, Byte, $value)
    }
  );
);

#[macro_export]
macro_rules! s_u16 (
  ($writer:ident!($($wargs:tt)*), $config:expr, $value:expr) => (
    {
      $writer!($($wargs)*, $config, Byte, ($value >> 8) as u8);
      $writer!($($wargs)*, $config, Byte, $value as u8);
    }
  );
);

#[macro_export]
macro_rules! s_u32 (
  ($writer:ident!($($wargs:tt)*), $config:expr, $value:expr) => (
    {
      $writer!($($wargs)*, $config, Byte, ($value >> 24) as u8);
      $writer!($($wargs)*, $config, Byte, ($value >> 16) as u8);
      $writer!($($wargs)*, $config, Byte, ($value >> 8) as u8);
      $writer!($($wargs)*, $config, Byte, $value as u8);
    }
  );
);

#[cfg(test)]
mod tests {
  use std::iter;

  #[test]
  fn writer_vec() {
    trace_macros!(true);
    let mut v = Vec::new();
    let res = s_u32!(write_vec!(v), (), 2147483647_u32);
    trace_macros!(false);
    println!("res: {:?}", res);
    println!("vec: {:?}", v);
    assert_eq!(&v[..], &[0x7f, 0xff, 0xff, 0xff]);
  }

  #[test]
  fn writer_slice() {
    trace_macros!(true);
    let mut v   = Vec::with_capacity(5);
    v.extend(iter::repeat(0).take(5));
    let mut pos = 0usize;
    let res = s_u32!(write_slice!((&mut v[..], pos)), (), 2147483647_u32);
    trace_macros!(false);
    println!("res: {:?}", res);
    println!("vec: {:?}, pos: {}", v, pos);
    assert_eq!(&v[..], &[0x7f, 0xff, 0xff, 0xff]);
    assert!(false);
  }
}
