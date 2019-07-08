use gen::GenError;
use lib::std::io::Write;
use lib::std::io;

pub type GenResult<I> = Result<I, GenError>;

pub trait SerializeFn<I>: Fn(I) -> GenResult<I> {}

impl<I, F: Fn(I) -> GenResult<I>> SerializeFn<I> for F {}

pub trait Skip: Write {
    fn skip(self, s: usize) -> Result<Self, GenError> where Self: Sized;
}

pub struct WriteCounter<W>(W, u64);

impl<W: Write> WriteCounter<W> {
    pub fn new(w: W) -> Self {
        WriteCounter(w, 0)
    }

    pub fn position(&self) -> u64 {
        self.1
    }

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

/// writes a byte slice to the output
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

/// writes a byte slice to the output
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

/// writes an hex string to the output
#[cfg(feature = "std")]
pub fn hex<S: ::lib::std::fmt::UpperHex, W: Write>(data: S) -> impl SerializeFn<W> {
  move |mut out: W| {
    match write!(out, "{:X}", data) {
      Err(io) => Err(GenError::IoError(io)),
      Ok(())  => Ok(out)
    }
  }
}

/// skips over some input bytes
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

/// applies 2 serializers in sequence
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

/// applies a serializer if the condition is true
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

/// applies an iterator of serializers
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

/// applies an iterator of serializers
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

pub fn be_u8<W: Write>(i: u8) -> impl SerializeFn<W> {
   let len = 1;

    move |mut out: W| {
        try_write!(out, len, &i.to_be_bytes()[..])
    }
}

pub fn be_u16<W: Write>(i: u16) -> impl SerializeFn<W> {
   let len = 2;

    move |mut out: W| {
        try_write!(out, len, &i.to_be_bytes()[..])
    }
}

pub fn be_u24<W: Write>(i: u32) -> impl SerializeFn<W> {
   let len = 3;

    move |mut out: W| {
        try_write!(out, len, &i.to_be_bytes()[1..])
    }
}

pub fn be_u32<W: Write>(i: u32) -> impl SerializeFn<W> {
   let len = 4;

    move |mut out: W| {
        try_write!(out, len, &i.to_be_bytes()[..])
    }
}

pub fn be_u64<W: Write>(i: u64) -> impl SerializeFn<W> {
   let len = 8;

    move |mut out: W| {
        try_write!(out, len, &i.to_be_bytes()[..])
    }
}

pub fn be_i8<W: Write>(i: i8) -> impl SerializeFn<W> {
    be_u8(i as u8)
}

pub fn be_i16<W: Write>(i: i16) -> impl SerializeFn<W> {
    be_u16(i as u16)
}

pub fn be_i24<W: Write>(i: i32) -> impl SerializeFn<W> {
    be_u24(i as u32)
}

pub fn be_i32<W: Write>(i: i32) -> impl SerializeFn<W> {
    be_u32(i as u32)
}

pub fn be_i64<W: Write>(i: i64) -> impl SerializeFn<W> {
    be_u64(i as u64)
}

pub fn be_f32<W: Write>(i: f32) -> impl SerializeFn<W> {
    be_u32(i.to_bits())
}

pub fn be_f64<W: Write>(i: f64) -> impl SerializeFn<W> {
    be_u64(i.to_bits())
}

pub fn le_u8<W: Write>(i: u8) -> impl SerializeFn<W> {
   let len = 1;

    move |mut out: W| {
        try_write!(out, len, &i.to_le_bytes()[..])
    }
}

pub fn le_u16<W: Write>(i: u16) -> impl SerializeFn<W> {
   let len = 2;

    move |mut out: W| {
        try_write!(out, len, &i.to_le_bytes()[..])
    }
}

pub fn le_u24<W: Write>(i: u32) -> impl SerializeFn<W> {
   let len = 3;

    move |mut out: W| {
        try_write!(out, len, &i.to_le_bytes()[1..])
    }
}

pub fn le_u32<W: Write>(i: u32) -> impl SerializeFn<W> {
   let len = 4;

    move |mut out: W| {
        try_write!(out, len, &i.to_le_bytes()[..])
    }
}

pub fn le_u64<W: Write>(i: u64) -> impl SerializeFn<W> {
   let len = 8;

    move |mut out: W| {
        try_write!(out, len, &i.to_le_bytes()[..])
    }
}

pub fn le_i8<W: Write>(i: i8) -> impl SerializeFn<W> {
    le_u8(i as u8)
}

pub fn le_i16<W: Write>(i: i16) -> impl SerializeFn<W> {
    le_u16(i as u16)
}

pub fn le_i24<W: Write>(i: i32) -> impl SerializeFn<W> {
    le_u24(i as u32)
}

pub fn le_i32<W: Write>(i: i32) -> impl SerializeFn<W> {
    le_u32(i as u32)
}

pub fn le_i64<W: Write>(i: i64) -> impl SerializeFn<W> {
    le_u64(i as u64)
}

pub fn le_f32<W: Write>(i: f32) -> impl SerializeFn<W> {
    le_u32(i.to_bits())
}

pub fn le_f64<W: Write>(i: f64) -> impl SerializeFn<W> {
    le_u64(i.to_bits())
}

/// applies a generator over an iterator of values, and applies the serializers generated
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

#[cfg(test)]
mod test {
    use super::*;

    #[cfg(feature = "std")]
    #[test]
    fn test_pair() {
        let mut buf = [0u8; 8];

        {
            use std::io::Cursor;

            let mut cursor = Cursor::new(&mut buf[..]);
            let serializer = pair(
                string("1234"),
                string("5678"),
            );

            let cursor = serializer(&mut cursor).unwrap();
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
