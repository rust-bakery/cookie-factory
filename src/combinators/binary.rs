use gen::GenError;
use super::{Serializer, Serialized, Reset};

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

pub struct LengthValue<A, B> {
  skip: usize,
  length: fn(usize) -> A,
  value: B,
}

impl<A:Serializer, B:Serializer> LengthValue<A, B> {
  #[inline(always)]
  pub fn new<U>(skip: U, length: fn(usize) -> A, value: B) -> Self
    where usize: From<U> {
    LengthValue {
      skip: usize::from(skip),
      length,
      value,
    }
  }
}

impl<A:Serializer, B:Serializer> Serializer for LengthValue<A,B> {
  #[inline(always)]
  fn serialize<'b, 'c>(&'b mut self, output: &'c mut [u8]) -> Result<(usize, Serialized), GenError> {
    match self.value.serialize(&mut output[self.skip..])? {
      // we need to write the full length/value structure at once
      (_, Serialized::Continue) => Ok((0, Serialized::Continue)),
      (i, Serialized::Done) => {
        match (self.length)(i).serialize(&mut output[..self.skip])? {
          (_, Serialized::Continue) => Err(GenError::InvalidOffset),
          (_, Serialized::Done) => Ok((self.skip+i, Serialized::Done))
        }
      }
    }
  }
}

pub fn length_value<A:Serializer, B:Serializer>(skip: usize, length: fn(usize) -> A, value: B) -> LengthValue<A, B> {
  LengthValue::new(skip, length, value)
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

#[cfg(test)]
mod tests {
  use super::*;
  use combinators::text::StrSr;

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

  #[test]
  fn length_value_test() {
      let mut mem : [u8; 100] = [0; 100];
      let expected = b"\x00\x05hello";

      let mut sr = length_value(2, |nb| (nb as u16).be(), "hello".raw());

      assert_eq!(sr.serialize(&mut mem), Ok((7, Serialized::Done)));
      assert_eq!(&mem[..7], expected);
  }
}
