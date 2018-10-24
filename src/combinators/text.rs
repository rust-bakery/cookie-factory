use std::str;
use super::binary::Slice;

pub trait StrSr {
  fn raw<'a>(&'a self) -> Slice<'a>;
}

impl<S: AsRef<str>> StrSr for S {
  #[inline(always)]
  fn raw<'a>(&'a self) -> Slice<'a> {
    Slice::new(self.as_ref().as_bytes())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use super::super::{Serializer, Serialized};

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

}
