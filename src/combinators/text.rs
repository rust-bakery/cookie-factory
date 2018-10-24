use std::str;
use std::fmt;
use std::io::{Write,Cursor};
use super::{Serializer, Serialized};
use super::binary::Slice;
use gen::GenError;

pub trait StrSr {
  fn raw<'a>(&'a self) -> Slice<'a>;
}

impl<S: AsRef<str>> StrSr for S {
  #[inline(always)]
  fn raw<'a>(&'a self) -> Slice<'a> {
    Slice::new(self.as_ref().as_bytes())
  }
}

pub struct Print<'a, T> {
  value: &'a T,
}

impl<'a, T: fmt::Display> Serializer for Print<'a, T> {
  fn serialize<'b, 'c>(&'b mut self, output: &'c mut [u8]) -> Result<(usize, Serialized), GenError> {
    let mut c = Cursor::new(output);
    match write!(&mut c, "{}", self.value) {
      //FIXME: maybe return an error here instead of assuming the buffer is too small?
      Err(_) => Ok((0, Serialized::Continue)),
      Ok(_) => Ok((c.position() as usize, Serialized::Done))
    }
  }
}

pub trait PrintSr<'a>: Sized {
  fn print(&'a self) -> Print<'a, Self>;
}

impl<'a, T: fmt::Display> PrintSr<'a> for T {
  #[inline(always)]
  fn print(&'a self) -> Print<'a, T> {
    Print { value: self }
  }
}

pub struct PrintUpperHex<T> {
  value: T,
}

impl<T: fmt::Display + fmt::UpperHex> Serializer for PrintUpperHex<T> {
  fn serialize<'b, 'c>(&'b mut self, output: &'c mut [u8]) -> Result<(usize, Serialized), GenError> {
    let mut c = Cursor::new(output);
    match write!(&mut c, "{:X}", self.value) {
      //FIXME: maybe return an error here instead of assuming the buffer is too small?
      Err(_) => Ok((0, Serialized::Continue)),
      Ok(_) => Ok((c.position() as usize, Serialized::Done))
    }
  }
}

pub trait PrintUpperHexSr: Sized + Copy {
  fn hex(self) -> PrintUpperHex<Self>;
}

impl<T: fmt::Display + fmt::UpperHex + Copy> PrintUpperHexSr for T {
  #[inline(always)]
  fn hex(self) -> PrintUpperHex<T> {
    PrintUpperHex { value: self }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use super::super::{Serializer, Serialized};
  use std::str::from_utf8;

  #[test]
  fn str_serializer() {
    let s = String::from("hello world!");
    let mut sr = Slice::new(s.as_str().as_bytes());

    let mut mem: [u8; 6] = [0; 6];
    let s = &mut mem[..];

    assert_eq!(sr.serialize(s), Ok((6, Serialized::Continue)));
    assert_eq!(&s[..], b"hello ");

    assert_eq!(sr.serialize(s), Ok((6, Serialized::Done)));
    assert_eq!(&s[..], b"world!");
  }

  #[test]
  fn print_serializer() {
    let mut mem: [u8; 100] = [0; 100];
    let s = &mut mem[..];

    let string = "hello";
    assert_eq!(string.print().serialize(s), Ok((5, Serialized::Done)));
    assert_eq!(from_utf8(&s[..5]).unwrap(), "hello");

    let num = 1234;
    assert_eq!(num.print().serialize(s), Ok((4, Serialized::Done)));
    assert_eq!(from_utf8(&s[..4]).unwrap(), "1234");

    let f = -1.2e14f32;
    assert_eq!(f.print().serialize(s), Ok((16, Serialized::Done)));
    assert_eq!(from_utf8(&s[..16]).unwrap(), "-120000000000000");

    let f2 = 1234.56e-2f64;
    assert_eq!(f2.print().serialize(s), Ok((7, Serialized::Done)));
    assert_eq!(from_utf8(&s[..7]).unwrap(), "12.3456");
  }

  #[test]
  fn print_hex_serializer() {
    let mut mem: [u8; 100] = [0; 100];
    let s = &mut mem[..];

    let num = 257;
    assert_eq!(num.hex().serialize(s), Ok((3, Serialized::Done)));
    assert_eq!(from_utf8(&s[..3]).unwrap(), "101");
  }
}
