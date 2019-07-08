use crate::gen::GenError;
use crate::lib::std::io::{self, Write};

/// Holds the result of serializing functions
///
/// The `Ok` case returns the `Write` used for writing, in the `Err` case an instance of
/// `cookie_factory::GenError` is returned.
pub type GenResult<I> = Result<I, GenError>;

/// Trait for serializing functions
///
/// Serializing functions take one input `I` that is the target of writing and return an instance
/// of `cookie_factory::GenResult`.
///
/// This trait is implemented for all `Fn(I) -> GenResult<I>`.
pub trait SerializeFn<I>: Fn(I) -> GenResult<I> {}

impl<I, F: Fn(I) -> GenResult<I>> SerializeFn<I> for F {}

/// Trait for `Write` types that allow skipping over the data
pub trait Skip: Write {
    fn skip(self, s: usize) -> Result<Self, GenError> where Self: Sized;
}

/// Wrapper around `Write` that counts how much data was written
///
/// This can be used to keep track how much data was actually written by serializing functions, for
/// example
///
/// ```rust
/// use cookie_factory::{WriteCounter, string};
///
/// let mut buf = [0u8; 100];
///
/// {
///     let mut writer = WriteCounter::new(&mut buf[..]);
///     writer = string("abcd")(writer).unwrap();
///     assert_eq!(writer.position(), 4);
///     let (buf, len) = writer.into_inner();
///     assert_eq!(buf.len(), 96);
///     assert_eq!(len, 4);
/// }
///
/// assert_eq!(&buf[..4], &b"abcd"[..]);
/// ```
///
/// For byte slices `std::io::Cursor` provides more features and allows to retrieve the original
/// slice
///
/// ```rust
/// #[cfg(feature = "std")]
/// use std::io::Cursor;
/// #[cfg(not(feature = "std"))]
/// use cookie_factory::lib::std::io::{Cursor, Write};
///
/// use cookie_factory::string;
///
/// let mut buf = [0u8; 100];
///
/// let mut cursor = Cursor::new(&mut buf[..]);
/// cursor = string("abcd")(cursor).unwrap();
/// assert_eq!(cursor.position(), 4);
/// let buf = cursor.into_inner();
///
/// assert_eq!(&buf[..4], &b"abcd"[..]);
/// ```
pub struct WriteCounter<W>(W, u64);

impl<W: Write> WriteCounter<W> {
    /// Create a new `WriteCounter` around `w`
    pub fn new(w: W) -> Self {
        WriteCounter(w, 0)
    }

    /// Returns the amount of bytes written so far
    pub fn position(&self) -> u64 {
        self.1
    }

    /// Consumes the `WriteCounter` and returns the contained `Write` and the amount of bytes
    /// written
    pub fn into_inner(self) -> (W, u64) {
        (self.0, self.1)
    }
}

impl<W: Write> Write for WriteCounter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let written = self.0.write(buf)?;
        self.1 += written as u64;
        Ok(written)
    }

    // Our minimal Write trait has no flush()
    #[cfg(feature = "std")]
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

macro_rules! try_write(($out:ident, $len:ident, $data:expr) => (
    match $out.write($data) {
        Err(io)           => Err(GenError::IoError(io)),
        Ok(n) if n < $len => Err(GenError::BufferTooSmall($len - n)),
        Ok(_)             => Ok($out)
    }
));

/// Writes a byte slice to the output
///
/// ```rust
/// use cookie_factory::slice;
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = slice(&b"abcd"[..])(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 4);
/// }
///
/// assert_eq!(&buf[..4], &b"abcd"[..]);
/// ```
pub fn slice<S: AsRef<[u8]>, W: Write>(data: S) -> impl SerializeFn<W> {
    let len = data.as_ref().len();

    move |mut out: W| {
        try_write!(out, len, data.as_ref())
    }
}

/// Writes a string slice to the output
///
/// ```rust
/// use cookie_factory::string;
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = string("abcd")(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 4);
/// }
///
/// assert_eq!(&buf[..4], &b"abcd"[..]);
/// ```
pub fn string<S: AsRef<str>, W: Write>(data: S) -> impl SerializeFn<W> {
    let len = data.as_ref().len();

    move |mut out: W| {
        try_write!(out, len, data.as_ref().as_bytes())
    }
}

/// Writes an hex string to the output
#[cfg(feature = "std")]
/// ```rust
/// use cookie_factory::hex;
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = hex(0x2A)(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 2);
/// }
///
/// assert_eq!(&buf[..2], &b"2A"[..]);
/// ```
pub fn hex<S: crate::lib::std::fmt::UpperHex, W: Write>(data: S) -> impl SerializeFn<W> {
  move |mut out: W| {
    match write!(out, "{:X}", data) {
      Err(io) => Err(GenError::IoError(io)),
      Ok(())  => Ok(out)
    }
  }
}

/// Skips over some input bytes without writing anything
///
/// ```rust
/// use cookie_factory::skip;
///
/// let mut buf = [0u8; 100];
///
/// let out = skip(2)(&mut buf[..]).unwrap();
///
/// assert_eq!(out.len(), 98);
/// ```
pub fn skip<W: Write + Skip>(len: usize) -> impl SerializeFn<W> {
    move |out: W| {
        out.skip(len)
    }
}

/// Applies 2 serializers in sequence
///
/// ```rust
/// use cookie_factory::{pair, string};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = pair(string("abcd"), string("efgh"))(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 8);
/// }
///
/// assert_eq!(&buf[..8], &b"abcdefgh"[..]);
/// ```
pub fn pair<F, G, I: Write>(first: F, second: G) -> impl SerializeFn<I>
where F: SerializeFn<I>,
      G: SerializeFn<I> {

  move |out: I| {
    first(out).and_then(&second)
  }
}

/// Applies a serializer if the condition is true
///
/// ```rust
/// use cookie_factory::{cond, string};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = cond(true, string("abcd"))(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 4);
/// }
///
/// assert_eq!(&buf[..4], &b"abcd"[..]);
/// ```
pub fn cond<F, I: Write>(condition: bool, f: F) -> impl SerializeFn<I>
where F: SerializeFn<I>, {

  move |out: I| {
    if condition {
      f(out)
    } else {
      Ok(out)
    }
  }
}

/// Applies an iterator of serializers of the same type
///
/// ```rust
/// use cookie_factory::{all, string};
///
/// let mut buf = [0u8; 100];
///
/// let data = vec!["abcd", "efgh", "ijkl"];
/// {
///   let buf = all(data.iter().map(string))(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 12);
/// }
///
/// assert_eq!(&buf[..12], &b"abcdefghijkl"[..]);
/// ```
pub fn all<G, I: Write, It>(values: It) -> impl SerializeFn<I>
  where G: SerializeFn<I>,
        It: Clone + Iterator<Item=G> {

  move |mut out: I| {
    let it = values.clone();

    for v in it {
      out = v(out)?;
    }

    Ok(out)
  }
}

/// Applies an iterator of serializers of the same type with a separator between each serializer
///
/// ```rust
/// use cookie_factory::{separated_list, string};
///
/// let mut buf = [0u8; 100];
///
/// let data = vec!["abcd", "efgh", "ijkl"];
/// {
///   let buf = separated_list(string(","), data.iter().map(string))(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 14);
/// }
///
/// assert_eq!(&buf[..14], &b"abcd,efgh,ijkl"[..]);
/// ```
pub fn separated_list<F, G, I: Write, It>(sep: F, values: It) -> impl SerializeFn<I>
  where F: SerializeFn<I>,
        G: SerializeFn<I>,
        It: Clone + Iterator<Item=G> {

  move |mut out: I| {
    let mut it = values.clone();

    match it.next() {
      None => return Ok(out),
      Some(first) => {
          out = first(out)?;
      }
    }

    for v in it {
      out = sep(out).and_then(v)?;
    }

    Ok(out)
  }
}

/// Writes an `u8` to the output
///
/// ```rust
/// use cookie_factory::be_u8;
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = be_u8(1u8)(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 1);
/// }
///
/// assert_eq!(&buf[..1], &[1u8][..]);
/// ```
pub fn be_u8<W: Write>(i: u8) -> impl SerializeFn<W> {
   let len = 1;

    move |mut out: W| {
        try_write!(out, len, &i.to_be_bytes()[..])
    }
}

/// Writes an `u16` in big endian byte order to the output
///
/// ```rust
/// use cookie_factory::be_u16;
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = be_u16(1u16)(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 2);
/// }
///
/// assert_eq!(&buf[..2], &[0u8, 1u8][..]);
/// ```
pub fn be_u16<W: Write>(i: u16) -> impl SerializeFn<W> {
   let len = 2;

    move |mut out: W| {
        try_write!(out, len, &i.to_be_bytes()[..])
    }
}

/// Writes the lower 24 bit of an `u32` in big endian byte order to the output
///
/// ```rust
/// use cookie_factory::be_u24;
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = be_u24(1u32)(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 3);
/// }
///
/// assert_eq!(&buf[..3], &[0u8, 0u8, 1u8][..]);
/// ```
pub fn be_u24<W: Write>(i: u32) -> impl SerializeFn<W> {
   let len = 3;

    move |mut out: W| {
        try_write!(out, len, &i.to_be_bytes()[1..])
    }
}

/// Writes an `u32` in big endian byte order to the output
///
/// ```rust
/// use cookie_factory::be_u32;
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = be_u32(1u32)(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 4);
/// }
///
/// assert_eq!(&buf[..4], &[0u8, 0u8, 0u8, 1u8][..]);
/// ```
pub fn be_u32<W: Write>(i: u32) -> impl SerializeFn<W> {
   let len = 4;

    move |mut out: W| {
        try_write!(out, len, &i.to_be_bytes()[..])
    }
}

/// Writes an `u64` in big endian byte order to the output
///
/// ```rust
/// use cookie_factory::be_u64;
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = be_u64(1u64)(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 8);
/// }
///
/// assert_eq!(&buf[..8], &[0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8][..]);
/// ```
pub fn be_u64<W: Write>(i: u64) -> impl SerializeFn<W> {
   let len = 8;

    move |mut out: W| {
        try_write!(out, len, &i.to_be_bytes()[..])
    }
}

/// Writes an `i8` to the output
///
/// ```rust
/// use cookie_factory::be_i8;
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = be_i8(1i8)(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 1);
/// }
///
/// assert_eq!(&buf[..1], &[1u8][..]);
/// ```
pub fn be_i8<W: Write>(i: i8) -> impl SerializeFn<W> {
    be_u8(i as u8)
}

/// Writes an `i16` in big endian byte order to the output
///
/// ```rust
/// use cookie_factory::be_i16;
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = be_i16(1i16)(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 2);
/// }
///
/// assert_eq!(&buf[..2], &[0u8, 1u8][..]);
/// ```
pub fn be_i16<W: Write>(i: i16) -> impl SerializeFn<W> {
    be_u16(i as u16)
}

/// Writes the lower 24 bit of an `i32` in big endian byte order to the output
///
/// ```rust
/// use cookie_factory::be_i24;
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = be_i24(1i32)(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 3);
/// }
///
/// assert_eq!(&buf[..3], &[0u8, 0u8, 1u8][..]);
/// ```
pub fn be_i24<W: Write>(i: i32) -> impl SerializeFn<W> {
    be_u24(i as u32)
}

/// Writes an `i32` in big endian byte order to the output
///
/// ```rust
/// use cookie_factory::be_i32;
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = be_i32(1i32)(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 4);
/// }
///
/// assert_eq!(&buf[..4], &[0u8, 0u8, 0u8, 1u8][..]);
/// ```
pub fn be_i32<W: Write>(i: i32) -> impl SerializeFn<W> {
    be_u32(i as u32)
}

/// Writes an `i64` in big endian byte order to the output
///
/// ```rust
/// use cookie_factory::be_i64;
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = be_i64(1i64)(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 8);
/// }
///
/// assert_eq!(&buf[..8], &[0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8][..]);
/// ```
pub fn be_i64<W: Write>(i: i64) -> impl SerializeFn<W> {
    be_u64(i as u64)
}

/// Writes an `f32` in big endian byte order to the output
///
/// ```rust
/// use cookie_factory::be_f32;
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = be_f32(1.0f32)(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 4);
/// }
///
/// assert_eq!(&buf[..4], &[63u8, 128u8, 0u8, 0u8][..]);
/// ```
pub fn be_f32<W: Write>(i: f32) -> impl SerializeFn<W> {
    be_u32(i.to_bits())
}

/// Writes an `f64` in big endian byte order to the output
///
/// ```rust
/// use cookie_factory::be_f64;
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = be_f64(1.0f64)(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 8);
/// }
///
/// assert_eq!(&buf[..8], &[63u8, 240u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8][..]);
/// ```
pub fn be_f64<W: Write>(i: f64) -> impl SerializeFn<W> {
    be_u64(i.to_bits())
}

/// Writes an `u8` to the output
///
/// ```rust
/// use cookie_factory::le_u8;
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = le_u8(1u8)(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 1);
/// }
///
/// assert_eq!(&buf[..1], &[1u8][..]);
/// ```
pub fn le_u8<W: Write>(i: u8) -> impl SerializeFn<W> {
   let len = 1;

    move |mut out: W| {
        try_write!(out, len, &i.to_le_bytes()[..])
    }
}

/// Writes an `u16` in little endian byte order to the output
///
/// ```rust
/// use cookie_factory::le_u16;
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = le_u16(1u16)(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 2);
/// }
///
/// assert_eq!(&buf[..2], &[1u8, 0u8][..]);
/// ```
pub fn le_u16<W: Write>(i: u16) -> impl SerializeFn<W> {
   let len = 2;

    move |mut out: W| {
        try_write!(out, len, &i.to_le_bytes()[..])
    }
}

/// Writes the lower 24 bit of an `u32` in little endian byte order to the output
///
/// ```rust
/// use cookie_factory::le_u24;
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = le_u24(1u32)(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 3);
/// }
///
/// assert_eq!(&buf[..3], &[1u8, 0u8, 0u8][..]);
/// ```
pub fn le_u24<W: Write>(i: u32) -> impl SerializeFn<W> {
   let len = 3;

    move |mut out: W| {
        try_write!(out, len, &i.to_le_bytes()[0..3])
    }
}

/// Writes an `u32` in little endian byte order to the output
///
/// ```rust
/// use cookie_factory::le_u32;
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = le_u32(1u32)(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 4);
/// }
///
/// assert_eq!(&buf[..4], &[1u8, 0u8, 0u8, 0u8][..]);
/// ```
pub fn le_u32<W: Write>(i: u32) -> impl SerializeFn<W> {
   let len = 4;

    move |mut out: W| {
        try_write!(out, len, &i.to_le_bytes()[..])
    }
}

/// Writes an `u64` in little endian byte order to the output
///
/// ```rust
/// use cookie_factory::le_u64;
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = le_u64(1u64)(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 8);
/// }
///
/// assert_eq!(&buf[..8], &[1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8][..]);
/// ```
pub fn le_u64<W: Write>(i: u64) -> impl SerializeFn<W> {
   let len = 8;

    move |mut out: W| {
        try_write!(out, len, &i.to_le_bytes()[..])
    }
}

/// Writes an `i8` to the output
///
/// ```rust
/// use cookie_factory::le_i8;
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = le_i8(1i8)(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 1);
/// }
///
/// assert_eq!(&buf[..1], &[1u8][..]);
/// ```
pub fn le_i8<W: Write>(i: i8) -> impl SerializeFn<W> {
    le_u8(i as u8)
}

/// Writes an `o16` in little endian byte order to the output
///
/// ```rust
/// use cookie_factory::le_i16;
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = le_i16(1i16)(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 2);
/// }
///
/// assert_eq!(&buf[..2], &[1u8, 0u8][..]);
/// ```
pub fn le_i16<W: Write>(i: i16) -> impl SerializeFn<W> {
    le_u16(i as u16)
}

/// Writes the lower 24 bit of an `i32` in little endian byte order to the output
///
/// ```rust
/// use cookie_factory::le_i24;
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = le_i24(1i32)(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 3);
/// }
///
/// assert_eq!(&buf[..3], &[1u8, 0u8, 0u8][..]);
/// ```
pub fn le_i24<W: Write>(i: i32) -> impl SerializeFn<W> {
    le_u24(i as u32)
}

/// Writes an `i32` in little endian byte order to the output
///
/// ```rust
/// use cookie_factory::le_i32;
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = le_i32(1i32)(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 4);
/// }
///
/// assert_eq!(&buf[..4], &[1u8, 0u8, 0u8, 0u8][..]);
/// ```
pub fn le_i32<W: Write>(i: i32) -> impl SerializeFn<W> {
    le_u32(i as u32)
}

/// Writes an `i64` in little endian byte order to the output
///
/// ```rust
/// use cookie_factory::le_i64;
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = le_i64(1i64)(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 8);
/// }
///
/// assert_eq!(&buf[..8], &[1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8][..]);
/// ```
pub fn le_i64<W: Write>(i: i64) -> impl SerializeFn<W> {
    le_u64(i as u64)
}

/// Writes an `f32` in little endian byte order to the output
///
/// ```rust
/// use cookie_factory::le_f32;
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = le_f32(1.0f32)(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 4);
/// }
///
/// assert_eq!(&buf[..4], &[0u8, 0u8, 128u8, 63u8][..]);
/// ```
pub fn le_f32<W: Write>(i: f32) -> impl SerializeFn<W> {
    le_u32(i.to_bits())
}

/// Writes an `f64` in little endian byte order to the output
///
/// ```rust
/// use cookie_factory::le_f64;
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = le_f64(1.0f64)(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 8);
/// }
///
/// assert_eq!(&buf[..8], &[0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 240u8, 63u8][..]);
/// ```
pub fn le_f64<W: Write>(i: f64) -> impl SerializeFn<W> {
    le_u64(i.to_bits())
}

/// Applies a generator over an iterator of values, and applies the serializers generated
///
/// ```rust
/// use cookie_factory::{many_ref, string};
///
/// let mut buf = [0u8; 100];
///
/// let data = vec!["abcd", "efgh", "ijkl"];
/// {
///   let buf = many_ref(&data, string)(&mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 12);
/// }
///
/// assert_eq!(&buf[..12], &b"abcdefghijkl"[..]);
/// ```
pub fn many_ref<E, It, I, F, G, O: Write>(items: I, generator: F) -> impl SerializeFn<O>
where
    It: Iterator<Item = E> + Clone,
    I: IntoIterator<Item = E, IntoIter = It>,
    F: Fn(E) -> G,
    G: SerializeFn<O>
{
    let items = items.into_iter();
    move |mut out: O| {
        for item in items.clone() {
            out = generator(item)(out)?;
        }
        Ok(out)
    }
}

//missing combinators:
//or
//empty
//then
//stream
//length_value
//text print
//text upperhex
//text lowerhex

impl Skip for &mut [u8] {
    fn skip(self, len: usize) -> Result<Self, GenError> {
        if self.len() < len {
            Err(GenError::BufferTooSmall(len - self.len()))
        } else {
            Ok(&mut self[len..])
        }
    }
}

impl<'a> Skip for io::Cursor<&'a mut [u8]> {
    fn skip(mut self, len: usize) -> Result<Self, GenError> {
        let remaining = self.get_ref().len().saturating_sub(self.position() as usize);
        if remaining < len {
            Err(GenError::BufferTooSmall(len - remaining))
        } else {
            self.set_position(self.position() + len as u64);
            Ok(self)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_pair_with_cursor() {
        let mut buf = [0u8; 8];

        {
            use self::io::Cursor;

            let cursor = Cursor::new(&mut buf[..]);
            let serializer = pair(
                string("1234"),
                string("5678"),
            );

            let cursor = serializer(cursor).unwrap();
            assert_eq!(cursor.position(), 8);
        }

        assert_eq!(&buf[..], b"12345678");
    }

    #[test]
    fn test_gen_with_length() {
        let mut buf = [0; 8];
        {
            let (len_buf, buf) = buf.split_at_mut(4);
            let w = WriteCounter::new(buf);
            let w = string("test")(w).unwrap();
            be_u32(w.position() as u32)(len_buf).unwrap();
        }
        assert_eq!(&buf, &[0, 0, 0, 4, 't' as u8, 'e' as u8, 's' as u8, 't' as u8]);
    }
}
