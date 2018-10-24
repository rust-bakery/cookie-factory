
use std::str;
use std::collections::HashMap;

use gen::GenError;

#[derive(Clone,Copy,Debug,PartialEq)]
pub enum Serialized {
  Done,
  Continue,
}

pub trait Serializer {
  fn serialize(&mut self, output: &mut [u8]) -> Result<(usize, Serialized), GenError>;

  #[inline(always)]
  fn then<T>(self, next: T) -> Then<Self, T>
    where
      Self: Sized,
      T: Serializer
  {
    Then::new(self, next)
  }

}


pub fn or<T,U>(t: Option<T>, u: U) -> Or<T, U>
  where
    T: Serializer,
    U: Serializer
{
  Or::new(t, u)
}

#[derive(Debug)]
pub struct Empty;

impl Serializer for Empty {
  #[inline(always)]
  fn serialize<'b, 'c>(&'b mut self, _output: &'c mut [u8]) -> Result<(usize, Serialized), GenError> {
    Ok((0, Serialized::Done))
  }
}

#[inline(always)]
pub fn empty() -> Empty {
  Empty
}

#[derive(Debug)]
pub struct Slice<'a> {
  value: &'a [u8],
}

impl<'a> Slice<'a> {
  #[inline(always)]
  pub fn new(s: &'a [u8]) -> Slice<'a> {
    Slice {
      value: s,
    }
  }
}

use std::ptr;
impl<'a> Serializer for Slice<'a> {
  #[inline(always)]
  fn serialize<'b, 'c>(&'b mut self, output: &'c mut [u8]) -> Result<(usize, Serialized), GenError> {
    let output_len = output.len();
    let self_len = self.value.len();
    if self_len <= output_len {
      (&mut output[..self_len]).copy_from_slice(self.value);
      Ok((self_len, Serialized::Done))
    } else {
      output.copy_from_slice(&self.value[..output_len]);
      self.value = &self.value[output_len..];
      Ok((output_len, Serialized::Continue))
    }
  }
}

impl<S: ?Sized + Serializer> Serializer for Box<S> {
  #[inline(always)]
  fn serialize<'b, 'c>(&'b mut self, output: &'c mut [u8]) -> Result<(usize, Serialized), GenError> {
    (**self).serialize(output)
  }
}

pub struct Then<A, B> {
  a: Option<A>,
  b: B,
}

impl<A:Serializer, B:Serializer> Then<A, B> {
  #[inline(always)]
  pub fn new(a: A, b: B) -> Self {
    Then {
      a: Some(a),
      b,
    }
  }
}

impl<A:Serializer, B:Serializer> Serializer for Then<A,B> {
  #[inline(always)]
  fn serialize<'b, 'c>(&'b mut self, output: &'c mut [u8]) -> Result<(usize, Serialized), GenError> {
    let mut i = 0;
    if let Some(mut a) = self.a.take() {
      match a.serialize(output)? {
        (index, Serialized::Continue) => {
          self.a = Some(a);
          return Ok((index, Serialized::Continue))
        },
        (index, Serialized::Done) => {
          i = index;
        }
      }
    }

    let sl = &mut output[i..];
    self.b.serialize(sl).map(|(index, res)| (index+i, res))
  }
}

pub struct Or<A, B> {
  a: Option<A>,
  b: B,
}

impl<A:Serializer, B:Serializer> Or<A, B> {
  #[inline(always)]
  pub fn new(a: Option<A>, b: B) -> Self {
    Or {
      a,
      b,
    }
  }
}

impl<A:Serializer, B:Serializer> Serializer for Or<A,B> {
  #[inline(always)]
  fn serialize<'b, 'c>(&'b mut self, output: &'c mut [u8]) -> Result<(usize, Serialized), GenError> {
    match &mut self.a {
      Some(ref mut a) => a.serialize(output),
      None => self.b.serialize(output)
    }
  }
}

pub struct All<T,It> {
  current: Option<T>,
  it: It,
}

impl<T: Serializer, It: Iterator<Item=T>> All<T, It> {
  #[inline(always)]
  pub fn new<IntoIt: IntoIterator<Item=T, IntoIter=It>>(it: IntoIt) -> Self {
    All {
      current: None,
      it: it.into_iter(),
    }
  }
}

impl<T: Serializer, It: Iterator<Item=T>> Serializer for All<T, It> {
  fn serialize<'b, 'c>(&'b mut self, output: &'c mut [u8]) -> Result<(usize, Serialized), GenError> {
    let mut index = 0;

    loop {
      let mut current = match self.current.take() {
        Some(s) => s,
        None => match self.it.next() {
          Some(s) => s,
          None => return Ok((index, Serialized::Done)),
        }
      };

      let sl = &mut output[index..];
      match current.serialize(sl)? {
        (i, Serialized::Continue) => {
          self.current = Some(current);
          return Ok((index + i, Serialized::Continue));
        },
        (i, Serialized::Done) => {
          index += i;
        },
      }
    }
  }
}
 
#[inline(always)]
pub fn all<T: Serializer, It: Iterator<Item=T>, IntoIt: IntoIterator<Item=T, IntoIter=It>>(it: IntoIt) -> All<T, It> {
  All::new(it)
}

pub trait StrSr {
  fn raw<'a>(&'a self) -> Slice<'a>;
}

impl<S: AsRef<str>> StrSr for S {
  #[inline(always)]
  fn raw<'a>(&'a self) -> Slice<'a> {
    Slice::new(self.as_ref().as_bytes())
  }
}

impl Serializer for Fn(&mut [u8]) -> Result<(&mut [u8],usize),GenError> {
  fn serialize<'b, 'c>(&'b mut self, output: &'c mut [u8]) -> Result<(usize, Serialized), GenError> {
    match self(output) {
      Err(e) => Err(e),
      Ok((_, index)) => Ok((index, Serialized::Done)),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

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
  fn then_serializer() {
    let s1 = String::from("hello ");
    let sr1 = Slice::new(s1.as_str().as_bytes());

    let s2 = String::from("world!");
    let sr2 = s2.raw();//StrSerializer::new(s2.as_str());

    let mut sr = sr1.then(sr2);

    let mut mem: [u8; 4] = [0; 4];
    let s = &mut mem[..];

    assert_eq!(sr.serialize(s), Ok((4, Serialized::Continue)));
    assert_eq!(&s[..], b"hell");

    assert_eq!(sr.serialize(s), Ok((4, Serialized::Continue)));
    assert_eq!(&s[..], b"o wo");

    assert_eq!(sr.serialize(s), Ok((4, Serialized::Done)));
    assert_eq!(&s[..], b"rld!");
  }
}
