use crate::gen::GenError;
use crate::lib::std::io::{self, SeekFrom, Write, Seek as _};

/// Holds the result of serializing functions
///
/// The `Ok` case returns the `Write` used for writing, in the `Err` case an instance of
/// `cookie_factory::GenError` is returned.
pub type GenResult<W> = Result<WriteContext<W>, GenError>;

/// Trait for serializing functions
///
/// Serializing functions take one input `W` that is the target of writing and return an instance
/// of `cookie_factory::GenResult`.
///
/// This trait is implemented for all `Fn(W) -> GenResult<W>`.
pub trait SerializeFn<W>: Fn(WriteContext<W>) -> GenResult<W> {}

impl<W, F: Fn(WriteContext<W>) -> GenResult<W>> SerializeFn<W> for F {}

/// Context around a `Write` impl that is passed through serializing functions
///
/// Currently this only keeps track of the current write position since the start of serialization.
pub struct WriteContext<W> {
    pub write: W,
    pub position: u64,
}

impl<W: Write> From<W> for WriteContext<W> {
    fn from(write: W) -> Self {
        Self {
            write,
            position: 0,
        }
    }
}

impl<W: Write> WriteContext<W> {
    /// Returns the contained `Write` and the current position
    pub fn into_inner(self) -> (W, u64) {
        (self.write, self.position)
    }
}

impl<W: Write> Write for WriteContext<W> {
    fn write(&mut self, data: &[u8]) -> crate::lib::std::io::Result<usize> {
        let amt = self.write.write(data)?;
        self.position += amt as u64;
        Ok(amt)
    }

    #[cfg(feature = "std")]
    fn flush(&mut self) -> io::Result<()> {
        self.write.flush()
    }
}

impl<W: Seek> io::Seek for WriteContext<W> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let old_pos = self.write.seek(SeekFrom::Current(0))?;
        let new_pos = self.write.seek(pos)?;
        if new_pos >= old_pos {
            self.position += new_pos - old_pos;
        } else {
            self.position -= old_pos - new_pos;
        }
        Ok(new_pos)
    }
}

/// Runs the given serializer `f` with the `Write` impl `w` and returns the result
///
/// This internally wraps `w` in a `WriteContext`, starting at position 0.
///
/// ```rust
/// use cookie_factory::{gen, slice};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(slice(&b"abcd"[..]), &mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 4);
///   assert_eq!(pos, 4);
/// }
///
/// assert_eq!(&buf[..4], &b"abcd"[..]);
/// ```
pub fn gen<W: Write, F: SerializeFn<W>>(f: F, w: W) -> Result<(W, u64), GenError> {
    f(WriteContext::from(w)).map(|ctx| ctx.into_inner())
}

/// Runs the given serializer `f` with the `Write` impl `w` and returns the updated `w`
///
/// This internally wraps `w` in a `WriteContext`, starting at position 0.
///
/// ```rust
/// use cookie_factory::{gen_simple, slice};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let buf = gen_simple(slice(&b"abcd"[..]), &mut buf[..]).unwrap();
///   assert_eq!(buf.len(), 100 - 4);
/// }
///
/// assert_eq!(&buf[..4], &b"abcd"[..]);
/// ```
pub fn gen_simple<W: Write, F: SerializeFn<W>>(f: F, w: W) -> Result<W, GenError> {
    f(WriteContext::from(w)).map(|ctx| ctx.into_inner().0)
}

/// Trait for `Write` types that allow skipping over the data
pub trait Skip: Write {
    fn skip(s: WriteContext<Self>, s: usize) -> GenResult<Self> where Self: Sized;
}

/// Trait for `Write` types that allow skipping and reserving a slice, then writing some data,
/// then write something in the slice we reserved using the return for our data write.
pub trait BackToTheBuffer: Write {
    fn reserve_write_use<Tmp, Gen: Fn(WriteContext<Self>) -> Result<(WriteContext<Self>, Tmp), GenError>, Before: Fn(WriteContext<Self>, &Tmp) -> GenResult<Self>>(s: WriteContext<Self>, reserved: usize, gen: &Gen, before: &Before) -> Result<(WriteContext<Self>, Tmp), GenError> where Self: Sized;
}

/// Trait for `Seek` types that want to automatically implement `BackToTheBuffer`
pub trait Seek: Write + io::Seek {}
impl Seek for io::Cursor<&mut [u8]> {}

impl<W: Seek> BackToTheBuffer for W {
    fn reserve_write_use<Tmp, Gen: Fn(WriteContext<Self>) -> Result<(WriteContext<Self>, Tmp), GenError>, Before: Fn(WriteContext<Self>, &Tmp) -> GenResult<Self>>(mut s: WriteContext<Self>, reserved: usize, gen: &Gen, before: &Before) -> Result<(WriteContext<Self>, Tmp), GenError> {
        let start = s.seek(SeekFrom::Current(0))?;
        let begin = s.seek(SeekFrom::Current(reserved as i64))?;
        let (mut buf, tmp) = gen(s)?;
        let end = buf.seek(SeekFrom::Current(0))?;
        buf.seek(SeekFrom::Start(start))?;
        let mut buf = before(buf, &tmp)?;
        let pos = buf.seek(SeekFrom::Current(0))?;
        if pos != begin {
            return Err(GenError::BufferTooBig((begin - pos) as usize));
        }
        buf.seek(SeekFrom::Start(end))?;
        Ok((buf, tmp))
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
/// use cookie_factory::{gen, slice};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(slice(&b"abcd"[..]), &mut buf[..]).unwrap();
///   assert_eq!(pos, 4);
///   assert_eq!(buf.len(), 100 - 4);
/// }
///
/// assert_eq!(&buf[..4], &b"abcd"[..]);
/// ```
pub fn slice<S: AsRef<[u8]>, W: Write>(data: S) -> impl SerializeFn<W> {
    let len = data.as_ref().len();

    move |mut out: WriteContext<W>| {
        try_write!(out, len, data.as_ref())
    }
}

/// Writes a string slice to the output
///
/// ```rust
/// use cookie_factory::{gen, string};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(string("abcd"), &mut buf[..]).unwrap();
///   assert_eq!(pos, 4);
///   assert_eq!(buf.len(), 100 - 4);
/// }
///
/// assert_eq!(&buf[..4], &b"abcd"[..]);
/// ```
pub fn string<S: AsRef<str>, W: Write>(data: S) -> impl SerializeFn<W> {
    let len = data.as_ref().len();

    move |mut out: WriteContext<W>| {
        try_write!(out, len, data.as_ref().as_bytes())
    }
}

/// Writes an hex string to the output
#[cfg(feature = "std")]
/// ```rust
/// use cookie_factory::{gen, hex};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(hex(0x2A), &mut buf[..]).unwrap();
///   assert_eq!(pos, 2);
///   assert_eq!(buf.len(), 100 - 2);
/// }
///
/// assert_eq!(&buf[..2], &b"2A"[..]);
/// ```
pub fn hex<S: crate::lib::std::fmt::UpperHex, W: Write>(data: S) -> impl SerializeFn<W> {
  move |mut out: WriteContext<W>| {
    match write!(out, "{:X}", data) {
      Err(io) => Err(GenError::IoError(io)),
      Ok(())  => Ok(out)
    }
  }
}

/// Skips over some input bytes without writing anything
///
/// ```rust
/// use cookie_factory::{gen, skip};
///
/// let mut buf = [0u8; 100];
///
/// let (out, pos) = gen(skip(2), &mut buf[..]).unwrap();
///
/// assert_eq!(pos, 2);
/// assert_eq!(out.len(), 98);
/// ```
pub fn skip<W: Write + Skip>(len: usize) -> impl SerializeFn<W> {
    move |out: WriteContext<W>| {
        W::skip(out, len)
    }
}

/// Applies 2 serializers in sequence
///
/// ```rust
/// use cookie_factory::{gen, pair, string};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(pair(string("abcd"), string("efgh")), &mut buf[..]).unwrap();
///   assert_eq!(pos, 8);
///   assert_eq!(buf.len(), 100 - 8);
/// }
///
/// assert_eq!(&buf[..8], &b"abcdefgh"[..]);
/// ```
pub fn pair<F, G, W: Write>(first: F, second: G) -> impl SerializeFn<W>
where F: SerializeFn<W>,
      G: SerializeFn<W> {

  move |out: WriteContext<W>| {
    first(out).and_then(&second)
  }
}

/// Applies a serializer if the condition is true
///
/// ```rust
/// use cookie_factory::{gen, cond, string};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(cond(true, string("abcd")), &mut buf[..]).unwrap();
///   assert_eq!(pos, 4);
///   assert_eq!(buf.len(), 100 - 4);
/// }
///
/// assert_eq!(&buf[..4], &b"abcd"[..]);
/// ```
pub fn cond<F, W: Write>(condition: bool, f: F) -> impl SerializeFn<W>
where F: SerializeFn<W>, {

  move |out: WriteContext<W>| {
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
/// use cookie_factory::{gen, all, string};
///
/// let mut buf = [0u8; 100];
///
/// let data = vec!["abcd", "efgh", "ijkl"];
/// {
///   let (buf, pos) = gen(all(data.iter().map(string)), &mut buf[..]).unwrap();
///   assert_eq!(pos, 12);
///   assert_eq!(buf.len(), 100 - 12);
/// }
///
/// assert_eq!(&buf[..12], &b"abcdefghijkl"[..]);
/// ```
pub fn all<G, W: Write, It>(values: It) -> impl SerializeFn<W>
  where G: SerializeFn<W>,
        It: Clone + Iterator<Item=G> {

  move |mut out: WriteContext<W>| {
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
/// use cookie_factory::{gen, separated_list, string};
///
/// let mut buf = [0u8; 100];
///
/// let data = vec!["abcd", "efgh", "ijkl"];
/// {
///   let (buf, pos) = gen(separated_list(string(","), data.iter().map(string)), &mut buf[..]).unwrap();
///   assert_eq!(pos, 14);
///   assert_eq!(buf.len(), 100 - 14);
/// }
///
/// assert_eq!(&buf[..14], &b"abcd,efgh,ijkl"[..]);
/// ```
pub fn separated_list<F, G, W: Write, It>(sep: F, values: It) -> impl SerializeFn<W>
  where F: SerializeFn<W>,
        G: SerializeFn<W>,
        It: Clone + Iterator<Item=G> {

  move |mut out: WriteContext<W>| {
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
/// use cookie_factory::{gen, be_u8};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(be_u8(1u8), &mut buf[..]).unwrap();
///   assert_eq!(pos, 1);
///   assert_eq!(buf.len(), 100 - 1);
/// }
///
/// assert_eq!(&buf[..1], &[1u8][..]);
/// ```
pub fn be_u8<W: Write>(i: u8) -> impl SerializeFn<W> {
   let len = 1;

    move |mut out: WriteContext<W>| {
        try_write!(out, len, &i.to_be_bytes()[..])
    }
}

/// Writes an `u16` in big endian byte order to the output
///
/// ```rust
/// use cookie_factory::{gen, be_u16};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(be_u16(1u16), &mut buf[..]).unwrap();
///   assert_eq!(pos, 2);
///   assert_eq!(buf.len(), 100 - 2);
/// }
///
/// assert_eq!(&buf[..2], &[0u8, 1u8][..]);
/// ```
pub fn be_u16<W: Write>(i: u16) -> impl SerializeFn<W> {
   let len = 2;

    move |mut out: WriteContext<W>| {
        try_write!(out, len, &i.to_be_bytes()[..])
    }
}

/// Writes the lower 24 bit of an `u32` in big endian byte order to the output
///
/// ```rust
/// use cookie_factory::{gen, be_u24};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(be_u24(1u32), &mut buf[..]).unwrap();
///   assert_eq!(pos, 3);
///   assert_eq!(buf.len(), 100 - 3);
/// }
///
/// assert_eq!(&buf[..3], &[0u8, 0u8, 1u8][..]);
/// ```
pub fn be_u24<W: Write>(i: u32) -> impl SerializeFn<W> {
   let len = 3;

    move |mut out: WriteContext<W>| {
        try_write!(out, len, &i.to_be_bytes()[1..])
    }
}

/// Writes an `u32` in big endian byte order to the output
///
/// ```rust
/// use cookie_factory::{gen, be_u32};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(be_u32(1u32), &mut buf[..]).unwrap();
///   assert_eq!(pos, 4);
///   assert_eq!(buf.len(), 100 - 4);
/// }
///
/// assert_eq!(&buf[..4], &[0u8, 0u8, 0u8, 1u8][..]);
/// ```
pub fn be_u32<W: Write>(i: u32) -> impl SerializeFn<W> {
   let len = 4;

    move |mut out: WriteContext<W>| {
        try_write!(out, len, &i.to_be_bytes()[..])
    }
}

/// Writes an `u64` in big endian byte order to the output
///
/// ```rust
/// use cookie_factory::{gen, be_u64};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(be_u64(1u64), &mut buf[..]).unwrap();
///   assert_eq!(pos, 8);
///   assert_eq!(buf.len(), 100 - 8);
/// }
///
/// assert_eq!(&buf[..8], &[0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8][..]);
/// ```
pub fn be_u64<W: Write>(i: u64) -> impl SerializeFn<W> {
   let len = 8;

    move |mut out: WriteContext<W>| {
        try_write!(out, len, &i.to_be_bytes()[..])
    }
}

/// Writes an `i8` to the output
///
/// ```rust
/// use cookie_factory::{gen, be_i8};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(be_i8(1i8), &mut buf[..]).unwrap();
///   assert_eq!(pos, 1);
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
/// use cookie_factory::{gen, be_i16};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(be_i16(1i16), &mut buf[..]).unwrap();
///   assert_eq!(pos, 2);
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
/// use cookie_factory::{gen, be_i24};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(be_i24(1i32), &mut buf[..]).unwrap();
///   assert_eq!(pos, 3);
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
/// use cookie_factory::{gen, be_i32};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(be_i32(1i32), &mut buf[..]).unwrap();
///   assert_eq!(pos, 4);
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
/// use cookie_factory::{gen, be_i64};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(be_i64(1i64), &mut buf[..]).unwrap();
///   assert_eq!(pos, 8);
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
/// use cookie_factory::{gen, be_f32};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(be_f32(1.0f32), &mut buf[..]).unwrap();
///   assert_eq!(pos, 4);
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
/// use cookie_factory::{gen, be_f64};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(be_f64(1.0f64), &mut buf[..]).unwrap();
///   assert_eq!(pos, 8);
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
/// use cookie_factory::{gen, le_u8};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(le_u8(1u8), &mut buf[..]).unwrap();
///   assert_eq!(pos, 1);
///   assert_eq!(buf.len(), 100 - 1);
/// }
///
/// assert_eq!(&buf[..1], &[1u8][..]);
/// ```
pub fn le_u8<W: Write>(i: u8) -> impl SerializeFn<W> {
   let len = 1;

    move |mut out: WriteContext<W>| {
        try_write!(out, len, &i.to_le_bytes()[..])
    }
}

/// Writes an `u16` in little endian byte order to the output
///
/// ```rust
/// use cookie_factory::{gen, le_u16};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(le_u16(1u16), &mut buf[..]).unwrap();
///   assert_eq!(pos, 2);
///   assert_eq!(buf.len(), 100 - 2);
/// }
///
/// assert_eq!(&buf[..2], &[1u8, 0u8][..]);
/// ```
pub fn le_u16<W: Write>(i: u16) -> impl SerializeFn<W> {
   let len = 2;

    move |mut out: WriteContext<W>| {
        try_write!(out, len, &i.to_le_bytes()[..])
    }
}

/// Writes the lower 24 bit of an `u32` in little endian byte order to the output
///
/// ```rust
/// use cookie_factory::{gen, le_u24};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(le_u24(1u32), &mut buf[..]).unwrap();
///   assert_eq!(pos, 3);
///   assert_eq!(buf.len(), 100 - 3);
/// }
///
/// assert_eq!(&buf[..3], &[1u8, 0u8, 0u8][..]);
/// ```
pub fn le_u24<W: Write>(i: u32) -> impl SerializeFn<W> {
   let len = 3;

    move |mut out: WriteContext<W>| {
        try_write!(out, len, &i.to_le_bytes()[0..3])
    }
}

/// Writes an `u32` in little endian byte order to the output
///
/// ```rust
/// use cookie_factory::{gen, le_u32};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(le_u32(1u32), &mut buf[..]).unwrap();
///   assert_eq!(pos, 4);
///   assert_eq!(buf.len(), 100 - 4);
/// }
///
/// assert_eq!(&buf[..4], &[1u8, 0u8, 0u8, 0u8][..]);
/// ```
pub fn le_u32<W: Write>(i: u32) -> impl SerializeFn<W> {
   let len = 4;

    move |mut out: WriteContext<W>| {
        try_write!(out, len, &i.to_le_bytes()[..])
    }
}

/// Writes an `u64` in little endian byte order to the output
///
/// ```rust
/// use cookie_factory::{gen, le_u64};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(le_u64(1u64), &mut buf[..]).unwrap();
///   assert_eq!(pos, 8);
///   assert_eq!(buf.len(), 100 - 8);
/// }
///
/// assert_eq!(&buf[..8], &[1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8][..]);
/// ```
pub fn le_u64<W: Write>(i: u64) -> impl SerializeFn<W> {
   let len = 8;

    move |mut out: WriteContext<W>| {
        try_write!(out, len, &i.to_le_bytes()[..])
    }
}

/// Writes an `i8` to the output
///
/// ```rust
/// use cookie_factory::{gen, le_i8};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(le_i8(1i8), &mut buf[..]).unwrap();
///   assert_eq!(pos, 1);
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
/// use cookie_factory::{gen, le_i16};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(le_i16(1i16), &mut buf[..]).unwrap();
///   assert_eq!(pos, 2);
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
/// use cookie_factory::{gen, le_i24};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(le_i24(1i32), &mut buf[..]).unwrap();
///   assert_eq!(pos, 3);
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
/// use cookie_factory::{gen, le_i32};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(le_i32(1i32), &mut buf[..]).unwrap();
///   assert_eq!(pos, 4);
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
/// use cookie_factory::{gen, le_i64};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(le_i64(1i64), &mut buf[..]).unwrap();
///   assert_eq!(pos, 8);
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
/// use cookie_factory::{gen, le_f32};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(le_f32(1.0f32), &mut buf[..]).unwrap();
///   assert_eq!(pos, 4);
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
/// use cookie_factory::{gen, le_f64};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(le_f64(1.0f64), &mut buf[..]).unwrap();
///   assert_eq!(pos, 8);
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
/// use cookie_factory::{gen, many_ref, string};
///
/// let mut buf = [0u8; 100];
///
/// let data = vec!["abcd", "efgh", "ijkl"];
/// {
///   let (buf, pos) = gen(many_ref(&data, string), &mut buf[..]).unwrap();
///   assert_eq!(pos, 12);
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
    move |mut out: WriteContext<O>| {
        for item in items.clone() {
            out = generator(item)(out)?;
        }
        Ok(out)
    }
}

/// Helper trait for the `tuple` combinator
pub trait Tuple<W> {
    fn serialize(&self, w: WriteContext<W>) -> GenResult<W>;
}

// Generates all the Tuple impls for tuples of arbitrary sizes based on a list of type
// parameters like FnA FnB FnC. It would generate the impl then for (FnA, FnB)
// and (FnA, FnB, FnC).
macro_rules! tuple_trait(
  ($name1:ident, $name2: ident, $($name:ident),*) => (
    tuple_trait!(__impl $name1, $name2; $($name),*);
  );
  (__impl $($name:ident),+; $name1:ident, $($name2:ident),*) => (
    tuple_trait_impl!($($name),+);
    tuple_trait!(__impl $($name),+ , $name1; $($name2),*);
  );
  (__impl $($name:ident),+; $name1:ident) => (
    tuple_trait_impl!($($name),+);
    tuple_trait_impl!($($name),+, $name1);
  );
);

// Generates the impl block for Tuple on tuples or arbitrary sizes based on its
// arguments. Takes a list of type parameters as parameters, e.g. FnA FnB FnC
// and then implements the trait on (FnA, FnB, FnC).
macro_rules! tuple_trait_impl(
  ($($name:ident),+) => (
    impl<W: Write, $($name: SerializeFn<W>),+> Tuple<W> for ( $($name),+ ) {
      fn serialize(&self, w: WriteContext<W>) -> GenResult<W> {
        tuple_trait_inner!(0, self, w, $($name)+)
      }
    }
  );
);

// Generates the inner part of the Tuple::serialize() implementation, which will
// basically look as follows:
//
// let w = self.0(w)?;
// let w = self.1(w)?;
// [...]
// let w = self.N(w)?;
//
// Ok(w)
macro_rules! tuple_trait_inner(
  ($it:tt, $self:expr, $w:ident, $head:ident $($id:ident)+) => ({
    let w = $self.$it($w)?;

    succ!($it, tuple_trait_inner!($self, w, $($id)+))
  });
  ($it:tt, $self:expr, $w:ident, $head:ident) => ({
    let w = $self.$it($w)?;

    Ok(w)
  });
);

// Takes an integer and a macro invocation, and changes the macro invocation
// to take the incremented integer as the first argument
//
// Works for integers between 0 and 19.
#[doc(hidden)]
macro_rules! succ (
  (0, $submac:ident ! ($($rest:tt)*)) => ($submac!(1, $($rest)*));
  (1, $submac:ident ! ($($rest:tt)*)) => ($submac!(2, $($rest)*));
  (2, $submac:ident ! ($($rest:tt)*)) => ($submac!(3, $($rest)*));
  (3, $submac:ident ! ($($rest:tt)*)) => ($submac!(4, $($rest)*));
  (4, $submac:ident ! ($($rest:tt)*)) => ($submac!(5, $($rest)*));
  (5, $submac:ident ! ($($rest:tt)*)) => ($submac!(6, $($rest)*));
  (6, $submac:ident ! ($($rest:tt)*)) => ($submac!(7, $($rest)*));
  (7, $submac:ident ! ($($rest:tt)*)) => ($submac!(8, $($rest)*));
  (8, $submac:ident ! ($($rest:tt)*)) => ($submac!(9, $($rest)*));
  (9, $submac:ident ! ($($rest:tt)*)) => ($submac!(10, $($rest)*));
  (10, $submac:ident ! ($($rest:tt)*)) => ($submac!(11, $($rest)*));
  (11, $submac:ident ! ($($rest:tt)*)) => ($submac!(12, $($rest)*));
  (12, $submac:ident ! ($($rest:tt)*)) => ($submac!(13, $($rest)*));
  (13, $submac:ident ! ($($rest:tt)*)) => ($submac!(14, $($rest)*));
  (14, $submac:ident ! ($($rest:tt)*)) => ($submac!(15, $($rest)*));
  (15, $submac:ident ! ($($rest:tt)*)) => ($submac!(16, $($rest)*));
  (16, $submac:ident ! ($($rest:tt)*)) => ($submac!(17, $($rest)*));
  (17, $submac:ident ! ($($rest:tt)*)) => ($submac!(18, $($rest)*));
  (18, $submac:ident ! ($($rest:tt)*)) => ($submac!(19, $($rest)*));
  (19, $submac:ident ! ($($rest:tt)*)) => ($submac!(20, $($rest)*));
);

tuple_trait!(FnA, FnB, FnC, FnD, FnE, FnF, FnG, FnH, FnI, FnJ, FnK, FnL,
FnM, FnN, FnO, FnP, FnQ, FnR, FnS, FnT, FnU);

/// Applies multiple serializers in sequence
///
/// Currently tuples up to 20 elements are supported.
///
/// ```rust
/// use cookie_factory::{gen, tuple, string, be_u16};
///
/// let mut buf = [0u8; 100];
///
/// {
///   let (buf, pos) = gen(
///     tuple((
///       string("abcd"),
///       be_u16(0x20),
///       string("efgh"),
///     )),
///     &mut buf[..]
///   ).unwrap();
///   assert_eq!(pos, 10);
///   assert_eq!(buf.len(), 100 - 10);
/// }
///
/// assert_eq!(&buf[..10], &b"abcd\x00\x20efgh"[..]);
/// ```
pub fn tuple<W: Write, List: Tuple<W>>(l: List) -> impl SerializeFn<W> {
    move |w: WriteContext<W>| {
        l.serialize(w)
    }
}

/// Reserves space for the `Before` combinator, applies the `Gen` combinator,
/// then applies the `Before` combinator with the output from `Gen` onto the
/// reserved space.
///
/// ```rust
/// use cookie_factory::{gen, gen_simple, tuple, back_to_the_buffer, string, be_u8, be_u32};
///
/// let mut buf = [0; 9];
/// gen_simple(tuple((
///     back_to_the_buffer(
///         4,
///         move |buf| gen(string("test"), buf),
///         move |buf, len| gen_simple(be_u32(*len as u32), buf)
///     ),
///     be_u8(42)
/// )), &mut buf[..]).unwrap();
/// assert_eq!(&buf, &[0, 0, 0, 4, 't' as u8, 'e' as u8, 's' as u8, 't' as u8, 42]);
/// ```
pub fn back_to_the_buffer<W: BackToTheBuffer, Tmp, Gen, Before>(reserved: usize, gen: Gen, before: Before) -> impl SerializeFn<W>
where Gen: Fn(WriteContext<W>) -> Result<(WriteContext<W>, Tmp), GenError>,
      Before: Fn(WriteContext<W>, &Tmp) -> GenResult<W> {
    move |w: WriteContext<W>| {
        W::reserve_write_use(w, reserved, &gen, &before).map(|t| t.0)
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
    fn skip(s: WriteContext<Self>, len: usize) -> Result<WriteContext<Self>, GenError> {
        if s.write.len() < len {
            Err(GenError::BufferTooSmall(len - s.write.len()))
        } else {
            Ok(WriteContext {
                write: &mut s.write[len..],
                position: s.position + len as u64,
            })
        }
    }
}

impl Skip for io::Cursor<&mut [u8]> {
    fn skip(mut s: WriteContext<Self>, len: usize) -> GenResult<Self> {
        let remaining = s.write.get_ref().len().saturating_sub(s.write.position() as usize);
        if remaining < len {
            Err(GenError::BufferTooSmall(len - remaining))
        } else {
            let cursor_position = s.write.position();
            s.write.set_position(cursor_position + len as u64);
            s.position += len as u64;
            Ok(s)
        }
    }
}

impl BackToTheBuffer for &mut [u8] {
    fn reserve_write_use<Tmp, Gen: Fn(WriteContext<Self>) -> Result<(WriteContext<Self>, Tmp), GenError>, Before: Fn(WriteContext<Self>, &Tmp) -> GenResult<Self>>(s: WriteContext<Self>, reserved: usize, gen: &Gen, before: &Before) -> Result<(WriteContext<Self>, Tmp), GenError> {
        let WriteContext { write: slice, position: original_position } = s;

        let (res, buf) = slice.split_at_mut(reserved);
        let (new_context, tmp) = gen(
          WriteContext {
            write: buf,
            position: original_position + reserved as u64
        })?;

        let res = before(
          WriteContext {
            write: res,
            position: original_position
          },
          &tmp,
        )?;

        if !res.write.is_empty() {
            return Err(GenError::BufferTooBig(res.write.len()));
        }

        Ok((new_context, tmp))
    }
}

#[cfg(feature = "std")]
impl BackToTheBuffer for Vec<u8> {
    fn reserve_write_use<Tmp, Gen: Fn(WriteContext<Self>) -> Result<(WriteContext<Self>, Tmp), GenError>, Before: Fn(WriteContext<Self>, &Tmp) -> GenResult<Self>>(s: WriteContext<Self>, reserved: usize, gen: &Gen, before: &Before) -> Result<(WriteContext<Self>, Tmp), GenError> {
        let WriteContext { write: mut vec, position: original_position } = s;

        let start_len = vec.len();
        vec.extend(std::iter::repeat(0).take(reserved));

        let (mut new_context, tmp) = gen(
          WriteContext {
            write: vec,
            position: original_position + reserved as u64,
        })?;

        let tmp_context = before(
          WriteContext {
            write: Vec::new(),
            position: original_position,
          },
          &tmp,
        )?;

        let tmp_written = tmp_context.write.len();
        if tmp_written != reserved {
            return Err(GenError::BufferTooBig(reserved - tmp_written));
        }

        // FIXME?: find a way to do that without copying
        // Vec::from_raw_parts + core::mem::forget makes it work, but
        // if `before` writes more than `reserved`, realloc will cause troubles
        new_context.write[start_len..(start_len + reserved)].copy_from_slice(&tmp_context.write[..]);

        Ok((new_context, tmp))
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

            let cursor = gen_simple(serializer, cursor).unwrap();
            assert_eq!(cursor.position(), 8);
        }

        assert_eq!(&buf[..], b"12345678");
    }

    #[test]
    fn test_tuple() {
        let mut buf = [0u8; 12];

        {
            use self::io::Cursor;

            let cursor = Cursor::new(&mut buf[..]);
            let serializer = tuple((
                string("1234"),
                string("5678"),
                string("0123"),
            ));

            let cursor = gen_simple(serializer, cursor).unwrap();
            assert_eq!(cursor.position(), 12);
        }

        assert_eq!(&buf[..], b"123456780123");
    }

    #[test]
    fn test_gen_with_length() {
        let mut buf = [0; 8];
        {
            let (len_buf, buf) = buf.split_at_mut(4);
            let (_, pos) = gen(string("test"), buf).unwrap();
            gen(be_u32(pos as u32), len_buf).unwrap();
        }
        assert_eq!(&buf, &[0, 0, 0, 4, 't' as u8, 'e' as u8, 's' as u8, 't' as u8]);
    }

    #[test]
    fn test_back_to_the_buffer() {
        let mut buf = [0; 9];

        let new_buf = gen_simple(
          tuple((
            back_to_the_buffer(
              4,
              move |buf| gen(string("test"), buf),
              move |buf, len| gen_simple(be_u32(*len as u32), buf)
            ),
            be_u8(42)
          )),
          &mut buf[..],
        ).unwrap();

        assert!(new_buf.is_empty());
        assert_eq!(&buf, &[0, 0, 0, 4, 't' as u8, 'e' as u8, 's' as u8, 't' as u8, 42]);
    }

    #[cfg(feature = "std")]
    #[test]
    fn test_back_to_the_buffer_vec() {
        let buf = Vec::new();

        let buf = gen_simple(
          tuple((
            back_to_the_buffer(
              4,
              move |buf| gen(string("test"), buf),
              move |buf, len| gen_simple(be_u32(*len as u32), buf)
            ),
            be_u8(42)
          )),
          buf,
        ).unwrap();

        assert_eq!(&buf[..], &[0, 0, 0, 4, 't' as u8, 'e' as u8, 's' as u8, 't' as u8, 42]);
    }

    #[test]
    fn test_back_to_the_buffer_cursor() {
        let mut buf = [0; 9];
        {
            let cursor = io::Cursor::new(&mut buf[..]);
            let cursor = gen_simple(
              tuple((
                back_to_the_buffer(
                  4,
                  move |buf| gen(string("test"), buf),
                  move |buf, len| gen_simple(be_u32(*len as u32), buf)
                ),
                be_u8(42)
              )),
              cursor,
            ).unwrap();
            assert_eq!(cursor.position(), 9);
        }
        assert_eq!(&buf, &[0, 0, 0, 4, 't' as u8, 'e' as u8, 's' as u8, 't' as u8, 42]);
    }

    #[test]
    fn test_back_to_the_buffer_cursor_counter() {
        let mut buf = [0; 10];
        {
            let cursor = io::Cursor::new(&mut buf[..]);
            let (cursor, pos) = gen(
              tuple((
                be_u8(64),
                back_to_the_buffer(
                  4,
                  &move |buf| gen(string("test"), buf),
                  &move |buf, len: &u64| gen_simple(be_u32(*len as u32), buf)
                ),
                be_u8(42),
              )),
              cursor,
            ).unwrap();
            assert_eq!(pos, 10);
            assert_eq!(cursor.position(), 10);
        }
        assert_eq!(&buf, &[64, 0, 0, 0, 4, 't' as u8, 'e' as u8, 's' as u8, 't' as u8, 42]);
    }
}
