use std::str;
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

pub trait Reset {
  fn reset(&mut self) -> bool;
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

impl Reset for Empty {
  fn reset(&mut self) -> bool {
    true
  }
}

#[inline(always)]
pub fn empty() -> Empty {
  Empty
}

#[derive(Debug)]
pub struct Slice<'a> {
  value: &'a [u8],
  index: usize,
}

impl<'a> Slice<'a> {
  #[inline(always)]
  pub fn new(s: &'a [u8]) -> Slice<'a> {
    Slice {
      value: s,
      index: 0,
    }
  }
}

impl<'a> Serializer for Slice<'a> {
  #[inline(always)]
  fn serialize<'b, 'c>(&'b mut self, output: &'c mut [u8]) -> Result<(usize, Serialized), GenError> {
    let output_len = output.len();
    let self_len = self.value.len() - self.index;
    if self_len <= output_len {
      (&mut output[..self_len]).copy_from_slice(&self.value[self.index..]);
      Ok((self_len, Serialized::Done))
    } else {
      output.copy_from_slice(&self.value[self.index..output_len]);
      self.index += output_len;
      Ok((output_len, Serialized::Continue))
    }
  }
}

impl<'a> Reset for Slice<'a> {
  fn reset(&mut self) -> bool {
    self.index = 0;
    true
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

impl<A:Serializer+Reset, B:Serializer+Reset> Reset for Or<A, B> {
  fn reset(&mut self) -> bool {
    self.a.as_mut().map(|a| a.reset()).unwrap_or(self.b.reset())
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

pub struct SeparatedList<T,U,It> {
  current: Option<T>,
  it: It,
  separator: U,
  serialize_element: bool,
}

impl<T: Serializer, U: Serializer+Reset, It: Iterator<Item=T>> SeparatedList<T, U, It> {
  #[inline(always)]
  pub fn new<IntoIt: IntoIterator<Item=T, IntoIter=It>>(separator: U, it: IntoIt) -> Self {
    let mut it = it.into_iter();
    SeparatedList {
      current: it.next(),
      it,
      separator,
      serialize_element: true,
    }
  }
}

impl<T: Serializer, U: Serializer+Reset, It: Iterator<Item=T>> Serializer for SeparatedList<T, U, It> {
  fn serialize<'b, 'c>(&'b mut self, output: &'c mut [u8]) -> Result<(usize, Serialized), GenError> {
    let mut index = 0;

    loop {
      let sl = &mut output[index..];

      if self.serialize_element {
        let mut current = match self.current.take() {
          Some(s) => s,
          None => return Ok((index, Serialized::Done)),
        };

        match current.serialize(sl)? {
          (i, Serialized::Continue) => {
            self.current = Some(current);
            return Ok((index + i, Serialized::Continue));
          },
          (i, Serialized::Done) => {
            index += i;
          },
        }

        self.current = self.it.next();
        if self.current.is_some() {
          self.serialize_element = false;
        }
      } else {
        // serialize separator
        match self.separator.serialize(sl)? {
          (i, Serialized::Continue) => {
            return Ok((index + i, Serialized::Continue));
          },
          (i, Serialized::Done) => {
            index += i;
            self.serialize_element = true;
            self.separator.reset();
          },

        }
      }
    }
  }
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

pub struct BigEndian<N> {
  pub num: N,
}

pub struct LittleEndian<N> {
  pub num: N,
}

pub trait Num: Copy {
  fn be(&self) -> BigEndian<Self> {
    BigEndian {
      num: *self,
    }
  }

  fn le(&self) -> LittleEndian<Self> {
    LittleEndian {
      num: *self,
    }
  }
}

impl Num for u8 {
}
impl Num for u16 {
}
impl Num for u32 {
}
impl Num for u64 {
}

impl Serializer for BigEndian<u8> {
  fn serialize<'b, 'c>(&'b mut self, output: &'c mut [u8]) -> Result<(usize, Serialized), GenError> {
    if output.len() == 0 {
      Ok((0, Serialized::Continue))
    } else {
      output[0] = self.num;
      Ok((1, Serialized::Done))
    }
  }
}

impl Serializer for BigEndian<u16> {
  fn serialize<'b, 'c>(&'b mut self, output: &'c mut [u8]) -> Result<(usize, Serialized), GenError> {
    if output.len() < 2 {
      Ok((0, Serialized::Continue))
    } else {
      output[0] = ((self.num >>  8) & 0xff) as u8;
      output[1] = ((self.num      ) & 0xff) as u8;
      Ok((2, Serialized::Done))
    }
  }
}

impl Serializer for BigEndian<u32> {
  fn serialize<'b, 'c>(&'b mut self, output: &'c mut [u8]) -> Result<(usize, Serialized), GenError> {
    if output.len() < 4 {
      Ok((0, Serialized::Continue))
    } else {
      output[0] = ((self.num >> 24) & 0xff) as u8;
      output[1] = ((self.num >> 16) & 0xff) as u8;
      output[2] = ((self.num >>  8) & 0xff) as u8;
      output[3] = ((self.num      ) & 0xff) as u8;
      Ok((4, Serialized::Done))
    }
  }
}

impl Serializer for BigEndian<u64> {
  fn serialize<'b, 'c>(&'b mut self, output: &'c mut [u8]) -> Result<(usize, Serialized), GenError> {
    if output.len() < 8 {
      Ok((0, Serialized::Continue))
    } else {
      output[0] = ((self.num >> 56) & 0xff) as u8;
      output[1] = ((self.num >> 48) & 0xff) as u8;
      output[2] = ((self.num >> 40) & 0xff) as u8;
      output[3] = ((self.num >> 32) & 0xff) as u8;
      output[4] = ((self.num >> 24) & 0xff) as u8;
      output[5] = ((self.num >> 16) & 0xff) as u8;
      output[6] = ((self.num >>  8) & 0xff) as u8;
      output[7] = ((self.num      ) & 0xff) as u8;
      Ok((8, Serialized::Done))
    }
  }
}

impl Serializer for LittleEndian<u8> {
  fn serialize<'b, 'c>(&'b mut self, output: &'c mut [u8]) -> Result<(usize, Serialized), GenError> {
    if output.len() == 0 {
      Ok((0, Serialized::Continue))
    } else {
      output[0] = self.num;
      Ok((1, Serialized::Done))
    }
  }
}

impl Serializer for LittleEndian<u16> {
  fn serialize<'b, 'c>(&'b mut self, output: &'c mut [u8]) -> Result<(usize, Serialized), GenError> {
    if output.len() < 2 {
      Ok((0, Serialized::Continue))
    } else {
      output[0] = ((self.num      ) & 0xff) as u8;
      output[1] = ((self.num >>  8) & 0xff) as u8;
      Ok((2, Serialized::Done))
    }
  }
}

impl Serializer for LittleEndian<u32> {
  fn serialize<'b, 'c>(&'b mut self, output: &'c mut [u8]) -> Result<(usize, Serialized), GenError> {
    if output.len() < 4 {
      Ok((0, Serialized::Continue))
    } else {
      output[0] = ((self.num      ) & 0xff) as u8;
      output[1] = ((self.num >>  8) & 0xff) as u8;
      output[2] = ((self.num >> 16) & 0xff) as u8;
      output[3] = ((self.num >> 24) & 0xff) as u8;
      Ok((4, Serialized::Done))
    }
  }
}

impl Serializer for LittleEndian<u64> {
  fn serialize<'b, 'c>(&'b mut self, output: &'c mut [u8]) -> Result<(usize, Serialized), GenError> {
    if output.len() < 8 {
      Ok((0, Serialized::Continue))
    } else {
      output[0] = ((self.num      ) & 0xff) as u8;
      output[1] = ((self.num >>  8) & 0xff) as u8;
      output[2] = ((self.num >> 16) & 0xff) as u8;
      output[3] = ((self.num >> 24) & 0xff) as u8;
      output[4] = ((self.num >> 32) & 0xff) as u8;
      output[5] = ((self.num >> 40) & 0xff) as u8;
      output[6] = ((self.num >> 48) & 0xff) as u8;
      output[7] = ((self.num >> 56) & 0xff) as u8;
      Ok((8, Serialized::Done))
    }
  }
}

impl<U> Reset for BigEndian<U> {
  fn reset(&mut self) -> bool {
    true
  }
}

impl<U> Reset for LittleEndian<U> {
  fn reset(&mut self) -> bool {
    true
  }
}

pub fn be<N: Num>(n: N) -> BigEndian<N> {
  n.be()
}

pub fn le<N: Num>(n: N) -> LittleEndian<N> {
  n.le()
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

  #[test]
  fn separated_list() {
    let mut mem: [u8; 100] = [0; 100];
    let s = &mut mem[..];

    let mut empty_list = SeparatedList::new(", ".raw(), [].iter().map(|_: &u8| empty()));
    assert_eq!(empty_list.serialize(s), Ok((0, Serialized::Done)));

    let mut single_element_list = SeparatedList::new(", ".raw(), ["hello"].iter().map(|s| s.raw()));
    assert_eq!(single_element_list.serialize(s), Ok((5, Serialized::Done)));
    assert_eq!(from_utf8(&s[..5]).unwrap(), "hello");

    let mut three_element_list = SeparatedList::new(", ".raw(), ["hello", "world", "hello again"].iter().map(|s| s.raw()));
    assert_eq!(three_element_list.serialize(s), Ok((25, Serialized::Done)));
    assert_eq!(from_utf8(&s[..25]).unwrap(), "hello, world, hello again");

    let mut three_element_list_partial = SeparatedList::new(", ".raw(), ["hello", "world", "hello again"].iter().map(|s| s.raw()));
    assert_eq!(three_element_list_partial.serialize(&mut s[..6]), Ok((6, Serialized::Continue)));
    assert_eq!(from_utf8(&s[..6]).unwrap(), "hello,");
    assert_eq!(three_element_list_partial.serialize(&mut s[6..11]), Ok((5, Serialized::Continue)));
    assert_eq!(from_utf8(&s[..11]).unwrap(), "hello, worl");
    assert_eq!(three_element_list_partial.serialize(&mut s[11..14]), Ok((3, Serialized::Continue)));
    assert_eq!(from_utf8(&s[..14]).unwrap(), "hello, world, ");
    assert_eq!(three_element_list_partial.serialize(&mut s[14..20]), Ok((6, Serialized::Continue)));
    assert_eq!(from_utf8(&s[..20]).unwrap(), "hello, world, hello ");
    assert_eq!(three_element_list_partial.serialize(&mut s[20..]), Ok((5, Serialized::Done)));
    assert_eq!(from_utf8(&s[..26]).unwrap(), "hello, world, hello again\0");
  }

  #[test]
  fn be_u8() {
      let mut mem : [u8; 8] = [0; 8];
      let expected = [1, 0, 0, 0, 0, 0, 0, 0];
      assert_eq!(0x01u8.be().serialize(&mut mem), Ok((1, Serialized::Done)));
      assert_eq!(mem, expected);
  }

  #[test]
  fn be_u16() {
      let mut mem : [u8; 8] = [0; 8];
      let expected = [1, 2, 0, 0, 0, 0, 0, 0];
      assert_eq!(0x0102u16.be().serialize(&mut mem), Ok((2, Serialized::Done)));
      assert_eq!(mem, expected);
  }

  #[test]
  fn be_u32() {
      let mut mem : [u8; 8] = [0; 8];
      let expected = [1, 2, 3, 4, 0, 0, 0, 0];
      assert_eq!(0x01020304u32.be().serialize(&mut mem), Ok((4, Serialized::Done)));
      assert_eq!(mem, expected);
  }

  #[test]
  fn be_u64() {
      let mut mem : [u8; 8] = [0; 8];
      let expected = [1, 2, 3, 4, 5, 6, 7, 8];
      assert_eq!(0x0102030405060708u64.be().serialize(&mut mem), Ok((8, Serialized::Done)));
      assert_eq!(mem, expected);
  }

  #[test]
  fn le_u8() {
      let mut mem : [u8; 8] = [0; 8];
      let expected = [1, 0, 0, 0, 0, 0, 0, 0];
      assert_eq!(le(0x01u8).serialize(&mut mem), Ok((1, Serialized::Done)));
      assert_eq!(mem, expected);
  }

  #[test]
  fn le_u16() {
      let mut mem : [u8; 8] = [0; 8];
      let expected = [2, 1, 0, 0, 0, 0, 0, 0];
      assert_eq!(le(0x0102u16).serialize(&mut mem), Ok((2, Serialized::Done)));
      assert_eq!(mem, expected);
  }
  #[test]
  fn le_u32() {
      let mut mem : [u8; 8] = [0; 8];
      let expected = [4, 3, 2, 1, 0, 0, 0, 0];
      assert_eq!(le(0x01020304u32).serialize(&mut mem), Ok((4, Serialized::Done)));
      assert_eq!(mem, expected);
  }
  #[test]
  fn le_u64() {
      let mut mem : [u8; 8] = [0; 8];
      let expected = [8, 7, 6, 5, 4, 3, 2, 1];
      assert_eq!(le(0x0102030405060708u64).serialize(&mut mem), Ok((8, Serialized::Done)));
      assert_eq!(mem, expected);
  }
}
