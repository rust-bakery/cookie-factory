use cftrait::CookieFactory;
use gen::GenError;
use std::io::{Cursor, Write};
use std::fmt;

pub trait SerializeFn<I>: Fn(I) -> Result<I, GenError> {}

impl<I, F:  Fn(I) ->Result<I, GenError>> SerializeFn<I> for F {}

macro_rules! cookie_factory_serializer(
    ($t:tt, $be:ident) => (
        pub fn $be<'a>(i: $t) -> impl SerializeFn<&'a mut [u8]> {
            move |out: &'a mut [u8]| {
                i.serialize(out)
            }
        }
    );
    ($t:tt, $be:ident, $le:ident) => (
        cookie_factory_serializer!($t, $be);
        pub fn $le<'a>(i: $t) -> impl SerializeFn<&'a mut [u8]> {
            move |out: &'a mut [u8]| {
                i.serialize_le(out)
            }
        }
    );
);

cookie_factory_serializer!(u8,  be_u8,  le_u8);
cookie_factory_serializer!(u16, be_u16, le_u16);
cookie_factory_serializer!(u32, be_u32, le_u32);
cookie_factory_serializer!(u64, be_u64, le_u64);
cookie_factory_serializer!(i8,  be_i8,  le_i8);
cookie_factory_serializer!(i16, be_i16, le_i16);
cookie_factory_serializer!(i32, be_i32, le_i32);
cookie_factory_serializer!(i64, be_i64, le_i64);
cookie_factory_serializer!(f32, be_f32, le_f32);
cookie_factory_serializer!(f64, be_f64, le_f64);

/// writes a byte slice to the output
///
/// ```rust
/// use cookie_factory::{length, slice};
///
/// let mut buf = [0u8; 100];
///
/// let len = {
///   let (len, _) = length(slice(&b"abcd"[..]))(&mut buf[..]).unwrap();
///   len
/// };
///
/// assert_eq!(len, 4usize);
/// assert_eq!(&buf[..4], &b"abcd"[..]);
/// ```
pub fn slice<'a, 'b, S: 'b + AsRef<[u8]>>(data: S) -> impl SerializeFn<&'a mut [u8]> {
    move |out: &'a mut [u8]| {
        data.as_ref().serialize(out)
    }
}

/// writes a string to the output
///
/// ```rust
/// use cookie_factory::{length, string};
///
/// let mut buf = [0u8; 100];
///
/// let len = {
///   let (len, _) = length(string("abcd"))(&mut buf[..]).unwrap();
///   len
/// };
///
/// assert_eq!(len, 4usize);
/// assert_eq!(&buf[..4], &b"abcd"[..]);
/// ```
pub fn string<'a, S: 'a+AsRef<str>>(data: S) -> impl SerializeFn<&'a mut [u8]> {
    move |out: &'a mut [u8]| {
        data.as_ref().serialize(out)
    }
}

/// writes a string to the output
///
/// ```rust
/// use cookie_factory::{length, string};
///
/// let mut buf = [0u8; 100];
///
/// let len = {
///   let (len, _) = length(string("abcd"))(&mut buf[..]).unwrap();
///   len
/// };
///
/// assert_eq!(len, 4usize);
/// assert_eq!(&buf[..4], &b"abcd"[..]);
/// ```
pub fn hex<'a, S: 'a + fmt::UpperHex>(data: S) -> impl SerializeFn<&'a mut [u8]> {

  move |out: &'a mut [u8]| {
    let mut c = Cursor::new(out);
    match write!(&mut c, "{:X}", data) {
      Err(_) => Err(GenError::CustomError(42)),
      Ok(_) => {
        let pos = c.position() as usize;
        let out = c.into_inner();
        Ok(&mut out[pos..])
      }
    }
  }
}

/// skips over some input bytes
///
/// ```rust
/// use cookie_factory::{length, skip};
///
/// let mut buf = [0u8; 100];
///
/// let out = skip(2)(&mut buf).unwrap();
///
/// assert_eq!(out.len(), 98);
/// ```
pub fn skip<'a>(len: usize) -> impl SerializeFn<&'a mut [u8]> {

    move |out: &'a mut [u8]| {
        if out.len() < len {
            Err(GenError::BufferTooSmall(len))
        } else {
            Ok(&mut out[len..])
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
    let f = length(f);

    move |out: &'a mut [u8]| {
        let ptr = out.as_mut_ptr();
        let (len, out) = f(out)?;
        Ok((unsafe { std::slice::from_raw_parts_mut(ptr, len) }, out))
    }
}

/// applies 2 serializers in sequence
///
/// ```rust
/// use cookie_factory::{length, pair, string};
///
/// let mut buf = [0u8; 100];
///
/// let len = {
///   let (len, _) = length(pair(string("abcd"), string("efgh")))(&mut buf[..]).unwrap();
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
    let out = first(out)?;
    second(out)
  }
}

/// applies a serializer if the condition is true
///
/// ```rust
/// use cookie_factory::{length, cond, string};
///
/// let mut buf = [0u8; 100];
///
/// let len = {
///   let (len, _) = length(cond(true, string("abcd")))(&mut buf[..]).unwrap();
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
      Ok(out)
    }
  }
}

/// applies an iterator of serializers
///
/// ```rust
/// use cookie_factory::{length, all, string};
///
/// let mut buf = [0u8; 100];
///
/// let data = vec!["abcd", "efgh", "ijkl"];
/// let len = {
///   let (len, _) = length(all(data.iter().map(string)))(&mut buf[..]).unwrap();
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

    for v in it {
      out = v(out)?;
    }

    Ok(out)
  }
}

/// applies an iterator of serializers
///
/// ```rust
/// use cookie_factory::{length, separated_list, string};
///
/// let mut buf = [0u8; 100];
///
/// let data = vec!["abcd", "efgh", "ijkl"];
/// let len = {
///   let (len, _) = length(separated_list(string(","), data.iter().map(string)))(&mut buf[..]).unwrap();
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

    match it.next() {
      None => return Ok(out),
      Some(first) => {
        out = first(out)?;
      }
    }

    for v in it {
      out = sep(out)?;
      out = v(out)?;
    }

    Ok(out)
  }
}

pub fn be_u24<'a>(i: u32) -> impl SerializeFn<&'a mut [u8]> {
   let len = 3;

    move |out: &'a mut [u8]| {
        if out.len() < len {
            Err(GenError::BufferTooSmall(len))
        } else {
            out[0] = ((i >> 16) & 0xff) as u8;
            out[1] = ((i >>  8) & 0xff) as u8;
            out[2] = ((i      ) & 0xff) as u8;
            Ok(&mut out[len..])
        }
    }
}

pub fn be_i24<'a>(i: i32) -> impl SerializeFn<&'a mut [u8]> {
    be_u24(i as u32)
}

pub fn le_u24<'a>(i: u32) -> impl SerializeFn<&'a mut [u8]> {
   let len = 3;

    move |out: &'a mut [u8]| {
        if out.len() < len {
            Err(GenError::BufferTooSmall(len))
        } else {
            out[0] = ((i      ) & 0xff) as u8;
            out[1] = ((i >>  8) & 0xff) as u8;
            out[2] = ((i >> 16) & 0xff) as u8;
            Ok(&mut out[len..])
        }
    }
}

pub fn le_i24<'a>(i: i32) -> impl SerializeFn<&'a mut [u8]> {
    le_u24(i as u32)
}

/// applies a generator over an iterator of values, and applies the serializers generated
///
/// ```rust
/// use cookie_factory::{length, many_ref, string};
///
/// let mut buf = [0u8; 100];
///
/// let data = vec!["abcd", "efgh", "ijkl"];
/// let len = {
///   let (len, _) = length(many_ref(&data, string))(&mut buf[..]).unwrap();
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
        for item in items.clone() {
            out = generator(item)(out)?;
        }
        Ok(out)
    }
}

/// returns the length of the data that was written, along with the remaining data
///
/// ```rust
/// use cookie_factory::{length,string};
///
/// let mut buf = [0u8; 100];
///
/// let len = {
///   let (len, _) = length(string("abcd"))(&mut buf[..]).unwrap();
///   len
/// };
///
/// assert_eq!(len, 4usize);
/// assert_eq!(&buf[..4], &b"abcd"[..]);
/// ```
pub fn length<'a, F>(f: F) -> impl Fn(&'a mut [u8]) -> Result<(usize, &'a mut [u8]), GenError>
  where F: SerializeFn<&'a mut [u8]> {
  move |out: &'a mut [u8]| {
    let start = out.as_ptr() as usize;

    let out = f(out)?;

    let end = out.as_ptr() as usize;
    Ok((end - start, out))
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
