use gen::GenError;
use std::io::Write;
use std::fmt;

pub trait SerializeFn<I>: Fn(I) -> Result<(I, usize), GenError> {}

impl<I, F:  Fn(I) ->Result<(I, usize), GenError>> SerializeFn<I> for F {}

pub trait Length {
    fn length(&self) -> usize;
}

pub trait Skip {
    fn skip(mut self, s: usize) -> Self;
}

macro_rules! try_write(($out:ident, $len:ident, $data:expr) => (
    match $out.write($data) {
        Err(io)           => Err(GenError::IoError(io)),
        Ok(n) if n < $len => Err(GenError::BufferTooSmall($len)),
        Ok(n)             => Ok(($out, n))
    }
));

/// writes a byte slice to the output
///
/// ```rust
/// use cookie_factory::slice;
///
/// let mut buf = [0u8; 100];
///
/// let len = {
///   let (_, len) = slice(&b"abcd"[..])(&mut buf[..]).unwrap();
///   len
/// };
///
/// assert_eq!(len, 4usize);
/// assert_eq!(&buf[..4], &b"abcd"[..]);
/// ```
pub fn slice<'a, 'b, S: 'b + AsRef<[u8]>, W: Write>(data: S) -> impl SerializeFn<W> {
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
/// let len = {
///   let (_, len) = string("abcd")(&mut buf[..]).unwrap();
///   len
/// };
///
/// assert_eq!(len, 4usize);
/// assert_eq!(&buf[..4], &b"abcd"[..]);
/// ```
pub fn string<'a, S: 'a+AsRef<str>, W: Write>(data: S) -> impl SerializeFn<W> {
    let len = data.as_ref().len();

    move |mut out: W| {
        try_write!(out, len, data.as_ref().as_bytes())
    }
}

/// writes an hex string to the output
pub fn hex<'a, S: 'a + fmt::UpperHex, W: Write>(data: S) -> impl SerializeFn<W> {
  move |mut out: W| {
    match write!(out, "{:X}", data) {
      Err(io) => Err(GenError::IoError(io)),
      Ok(())  => Ok((out, 0 /* FIXME */))
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
/// let (out, _) = skip(2)(&mut buf[..]).unwrap();
///
/// assert_eq!(out.len(), 98);
/// ```
pub fn skip<'a, W: Length + Skip>(len: usize) -> impl SerializeFn<W> {
    move |out: W| {
        if out.length() < len {
            Err(GenError::BufferTooSmall(len))
        } else {
            Ok((out.skip(len), len))
        }
    }
}

/// applies a serializer then returns a tuple containing what was written and the remaining output buffer
///
/// ```rust
/// use cookie_factory::{position, string};
///
/// let mut buf = [0u8; 100];
///
/// let (written, remaining) = position(string("abcd"))(&mut buf[..]).unwrap();
///
/// assert_eq!(remaining.len(), 96);
/// assert_eq!(written, &b"abcd"[..]);
/// ```
pub fn position<'a, F>(f: F) -> impl Fn(&'a mut [u8]) -> Result<(&'a mut [u8], &'a mut [u8]), GenError>
  where F: SerializeFn<&'a mut [u8]> {
    move |out: &'a mut [u8]| {
        let ptr = out.as_mut_ptr();
        let (out, len) = f(out)?;
        Ok((unsafe { std::slice::from_raw_parts_mut(ptr, len) }, out))
    }
}

/// applies 2 serializers in sequence
///
/// ```rust
/// use cookie_factory::{pair, string};
///
/// let mut buf = [0u8; 100];
///
/// let len = {
///   let (_, len) = pair(string("abcd"), string("efgh"))(&mut buf[..]).unwrap();
///   len
/// };
///
/// assert_eq!(len, 8usize);
/// assert_eq!(&buf[..8], &b"abcdefgh"[..]);
/// ```
pub fn pair<F, G, I>(first: F, second: G) -> impl SerializeFn<I>
where F: SerializeFn<I>,
      G: SerializeFn<I> {

  move |out: I| {
    let (out, len) = first(out)?;
    second(out).map(|(out, len2)| (out, len + len2))
  }
}

/// applies a serializer if the condition is true
///
/// ```rust
/// use cookie_factory::{cond, string};
///
/// let mut buf = [0u8; 100];
///
/// let len = {
///   let (_, len) = cond(true, string("abcd"))(&mut buf[..]).unwrap();
///   len
/// };
///
/// assert_eq!(len, 4usize);
/// assert_eq!(&buf[..4], &b"abcd"[..]);
/// ```
pub fn cond<F, I>(condition: bool, f: F) -> impl SerializeFn<I>
where F: SerializeFn<I>, {

  move |out: I| {
    if condition {
      f(out)
    } else {
      Ok((out, 0))
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
/// let len = {
///   let (_, len) = all(data.iter().map(string))(&mut buf[..]).unwrap();
///   len
/// };
///
/// assert_eq!(len, 12usize);
/// assert_eq!(&buf[..12], &b"abcdefghijkl"[..]);
/// ```
pub fn all<'a, 'b, G, I, It>(values: It) -> impl SerializeFn<I> + 'a
  where G: SerializeFn<I> + 'b,
        It: 'a + Clone + Iterator<Item=G> {

  move |mut out: I| {
    let it = values.clone();
    let mut len = 0;

    for v in it {
      let (_out, _len) = v(out)?;
      out = _out;
      len += _len;
    }

    Ok((out, len))
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
/// let len = {
///   let (_, len) = separated_list(string(","), data.iter().map(string))(&mut buf[..]).unwrap();
///   len
/// };
///
/// assert_eq!(len, 14usize);
/// assert_eq!(&buf[..14], &b"abcd,efgh,ijkl"[..]);
/// ```
pub fn separated_list<'a, 'b, 'c, F, G, I, It>(sep: F, values: It) -> impl SerializeFn<I> + 'a
  where F: SerializeFn<I> + 'b + 'a,
        G: SerializeFn<I> + 'c,
        It: 'a + Clone + Iterator<Item=G> {

  move |mut out: I| {
    let mut it = values.clone();
    let mut len = 0;

    match it.next() {
      None => return Ok((out, len)),
      Some(first) => {
        let (_out, _len) = first(out)?;
        out = _out;
        len += _len;
      }
    }

    for v in it {
      let (_out, _len) = sep(out)?;
      let (_out, _len2) = v(_out)?;
      out = _out;
      len += _len + _len2;
    }

    Ok((out, len))
  }
}

pub fn be_u8<'a, W: Write>(i: u8) -> impl SerializeFn<W> {
   let len = 1;

    move |mut out: W| {
        try_write!(out, len, &i.to_be_bytes()[..])
    }
}

pub fn be_u16<'a, W: Write>(i: u16) -> impl SerializeFn<W> {
   let len = 2;

    move |mut out: W| {
        try_write!(out, len, &i.to_be_bytes()[..])
    }
}

pub fn be_u24<'a, W: Write>(i: u32) -> impl SerializeFn<W> {
   let len = 3;

    move |mut out: W| {
        try_write!(out, len, &i.to_be_bytes()[1..])
    }
}

pub fn be_u32<'a, W: Write>(i: u32) -> impl SerializeFn<W> {
   let len = 4;

    move |mut out: W| {
        try_write!(out, len, &i.to_be_bytes()[..])
    }
}

pub fn be_u64<'a, W: Write>(i: u64) -> impl SerializeFn<W> {
   let len = 8;

    move |mut out: W| {
        try_write!(out, len, &i.to_be_bytes()[..])
    }
}

pub fn be_i8<'a, W: Write>(i: i8) -> impl SerializeFn<W> {
    be_u8(i as u8)
}

pub fn be_i16<'a, W: Write>(i: i16) -> impl SerializeFn<W> {
    be_u16(i as u16)
}

pub fn be_i24<'a, W: Write>(i: i32) -> impl SerializeFn<W> {
    be_u24(i as u32)
}

pub fn be_i32<'a, W: Write>(i: i32) -> impl SerializeFn<W> {
    be_u32(i as u32)
}

pub fn be_i64<'a, W: Write>(i: i64) -> impl SerializeFn<W> {
    be_u64(i as u64)
}

pub fn be_f32<'a, W: Write>(i: f32) -> impl SerializeFn<W> {
    be_u32(unsafe { std::mem::transmute::<f32, u32>(i) })
}

pub fn be_f64<'a, W: Write>(i: f64) -> impl SerializeFn<W> {
    be_u64(unsafe { std::mem::transmute::<f64, u64>(i) })
}

pub fn le_u8<'a, W: Write>(i: u8) -> impl SerializeFn<W> {
   let len = 1;

    move |mut out: W| {
        try_write!(out, len, &i.to_le_bytes()[..])
    }
}

pub fn le_u16<'a, W: Write>(i: u16) -> impl SerializeFn<W> {
   let len = 2;

    move |mut out: W| {
        try_write!(out, len, &i.to_le_bytes()[..])
    }
}

pub fn le_u24<'a, W: Write>(i: u32) -> impl SerializeFn<W> {
   let len = 3;

    move |mut out: W| {
        try_write!(out, len, &i.to_le_bytes()[1..])
    }
}

pub fn le_u32<'a, W: Write>(i: u32) -> impl SerializeFn<W> {
   let len = 4;

    move |mut out: W| {
        try_write!(out, len, &i.to_le_bytes()[..])
    }
}

pub fn le_u64<'a, W: Write>(i: u64) -> impl SerializeFn<W> {
   let len = 8;

    move |mut out: W| {
        try_write!(out, len, &i.to_le_bytes()[..])
    }
}

pub fn le_i8<'a, W: Write>(i: i8) -> impl SerializeFn<W> {
    le_u8(i as u8)
}

pub fn le_i16<'a, W: Write>(i: i16) -> impl SerializeFn<W> {
    le_u16(i as u16)
}

pub fn le_i24<'a, W: Write>(i: i32) -> impl SerializeFn<W> {
    le_u24(i as u32)
}

pub fn le_i32<'a, W: Write>(i: i32) -> impl SerializeFn<W> {
    le_u32(i as u32)
}

pub fn le_i64<'a, W: Write>(i: i64) -> impl SerializeFn<W> {
    le_u64(i as u64)
}

pub fn le_f32<'a, W: Write>(i: f32) -> impl SerializeFn<W> {
    le_u32(unsafe { std::mem::transmute::<f32, u32>(i) })
}

pub fn le_f64<'a, W: Write>(i: f64) -> impl SerializeFn<W> {
    le_u64(unsafe { std::mem::transmute::<f64, u64>(i) })
}

/// applies a generator over an iterator of values, and applies the serializers generated
///
/// ```rust
/// use cookie_factory::{many_ref, string};
///
/// let mut buf = [0u8; 100];
///
/// let data = vec!["abcd", "efgh", "ijkl"];
/// let len = {
///   let (_, len) = many_ref(&data, string)(&mut buf[..]).unwrap();
///   len
/// };
///
/// assert_eq!(len, 12usize);
/// assert_eq!(&buf[..12], &b"abcdefghijkl"[..]);
/// ```
pub fn many_ref<'a, E, It, I, F, G, O>(items: I, generator: F) -> impl SerializeFn<O> + 'a
where
    It: Iterator<Item = E> + Clone + 'a,
    I: IntoIterator<Item = E, IntoIter = It>,
    F: Fn(E) -> G + 'a,
    G: SerializeFn<O> + 'a,
    O: 'a
{
    let items = items.into_iter();
    move |mut out: O| {
        let mut len = 0;
        for item in items.clone() {
            let (_out, _len) = generator(item)(out)?;
            out = _out;
            len += _len;
        }
        Ok((out, len))
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

impl Length for &mut [u8] {
    fn length(&self) -> usize {
        self.len()
    }
}

impl Skip for &mut [u8] {
    fn skip(mut self, len: usize) -> Self {
        &mut self[len..]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_gen_with_length() {
        let mut buf = vec![0; 8];
        let start = buf.as_mut_ptr();
        let (len_buf, buf) = buf.split_at_mut(4);
        let (buf, len) = string("test")(buf).unwrap();
        be_u32(len as u32)(len_buf).unwrap();
        assert_eq!(buf, &mut []);
        assert_eq!(unsafe { std::slice::from_raw_parts_mut(start, 8) }, &[0, 0, 0, 4, 't' as u8, 'e' as u8, 's' as u8, 't' as u8]);
    }
}
