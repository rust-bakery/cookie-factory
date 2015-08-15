//! Cookie factory
#![feature(trace_macros)]
//macros:
//* write_vec!(vec, ...) -> Result(nb of bytes)
//* write_slice!(s, ...) -> Result(nb of bytes)
//* write_iterator!(it, ...) -> Result(nb of bytes)
//
//* config can be endianness
// combinator: c!(writer, config, conbinator/serializer)

#[macro_export]
macro_rules! write_vec (
  ($vec: expr,  $submac:ident!($($args:tt)*)) => (
    {
      $submac!(write_vec_impl!($vec), $($args)*)
    }
  )
);

#[macro_export]
macro_rules! write_vec_impl (
  ($vec:expr, $config:expr, Slice, $value: expr) => (
    {
      ($vec).extend(($value).iter().cloned());
      Ok(($value).len())
    }
  );
  ($vec:expr, $config:expr, Byte, $value: expr) => (
    {
      ($vec).push($value);
      let res:Result<usize,()> = Ok(1usize);
      res
    }
  );
);


#[macro_export]
macro_rules! write_slice (
  ($sl:expr, $submac:ident!($($args:tt)*)) => (
    {
      let mut pos = 0;
      $submac!(write_slice_impl!(($sl, pos)), $($args)*)
    }
  );
);


#[macro_export]
macro_rules! write_slice_impl (
  (($sl:expr, $pos:expr), $config:expr, Slice, $value: expr) => (
    {
      if ($value).len() < $sl.len() - $pos {
        std::slice::bytes::copy_memory($value, &mut ($sl)[$pos..($value.length)]);
        $pos += value.length;
        Ok(($value).len())
      } else {
        Err(())
      }
    }
  );
  (($sl:expr, $pos:expr), $config:expr, Byte, $value: expr) => (
    {
      if $sl.len() - $pos > 0 {
        $sl[$pos] = $value as u8;
        $pos += 1;
        Ok(1)
      } else {
        Err(())
      }
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
macro_rules! s_many (
  ($writer:ident!($($wargs:tt)*), $config:expr, $value:expr, $submac:ident!( $($args:tt)* )) => (
    {
      for el in $value {
        $submac:ident!( $writer!($($wargs)*), $config, $($args:tt)*, el )
      }
    }
  );
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
    let res:Result<usize,()> = write_vec!(v, s_u32!((), 2147483647_u32));
    trace_macros!(false);
    println!("res: {:?}", res);
    println!("vec: {:?}", v);
    assert_eq!(res, Ok(4));
    assert_eq!(&v[..], &[0x7f, 0xff, 0xff, 0xff]);
  }

  #[test]
  fn writer_slice() {
    trace_macros!(true);
    let mut v   = Vec::with_capacity(4);
    v.extend(iter::repeat(0).take(4));
    let res = write_slice!(&mut v[..], s_u32!((), 2147483647_u32));
    trace_macros!(false);
    println!("res: {:?}", res);
    println!("vec: {:?}", v);
    assert_eq!(res, Ok(4));
    assert_eq!(&v[..], &[0x7f, 0xff, 0xff, 0xff]);
  }

  #[test]
  fn writer_array() {
    trace_macros!(true);
    let mut v = Vec::new();
    let input = vec![0x7f, 0xff, 0xff, 0xff];

    let res = write_vec!(v, s_u32!((), 2147483647_u32));
    trace_macros!(false);
    println!("res: {:?}", res);
    println!("vec: {:?}", v);
    assert_eq!(res, Ok(4));
    assert_eq!(&v[..], &[0x7f, 0xff, 0xff, 0xff]);
    //assert!(false);
  }
}
