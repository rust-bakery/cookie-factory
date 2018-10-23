
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
pub struct EmptySerializer;

impl Serializer for EmptySerializer {
  #[inline(always)]
  fn serialize<'b, 'c>(&'c mut self, _output: &'b mut [u8]) -> Result<(usize, Serialized), GenError> {
    Ok((0, Serialized::Done))
  }
}

#[inline(always)]
pub fn empty() -> EmptySerializer {
  EmptySerializer
}

#[derive(Debug)]
pub struct SliceSerializer<'a> {
  value: &'a [u8],
}

impl<'a> SliceSerializer<'a> {
  #[inline(always)]
  pub fn new(s: &'a [u8]) -> SliceSerializer<'a> {
    SliceSerializer {
      value: s,
    }
  }
}

use std::ptr;
impl<'a> Serializer for SliceSerializer<'a> {
  #[inline(always)]
  fn serialize<'b, 'c>(&'c mut self, output: &'b mut [u8]) -> Result<(usize, Serialized), GenError> {
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
  fn serialize<'b, 'c>(&'c mut self, output: &'b mut [u8]) -> Result<(usize, Serialized), GenError> {
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
  fn serialize<'b, 'c>(&'c mut self, output: &'b mut [u8]) -> Result<(usize, Serialized), GenError> {
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

#[macro_export]
macro_rules! seq_ser (
  ($it:tt, $serializers:expr, $index:ident, $output:ident, $e:expr, $($elem:expr),*) => {
    if let Some(mut ser) = tup!($it, $serializers).take() {
      match ser.serialize(&mut $output[$index..])? {
        (i, Serialized::Continue) => {
          tup!($it, $serializers) = Some(ser);
          return Ok(($index + i, Serialized::Continue))
        },
        (i, Serialized::Done) => {
          $index += i;
        }
      }
    }

    succ!($it, seq_ser!($serializers, $index, $output, $($elem),*))
  };
  ($it:tt, $serializers:expr, $index:ident, $output:ident, $e:expr) => {
    if let Some(mut ser) = tup!($it, $serializers).take() {
      match ser.serialize(&mut $output[$index..])? {
        (i, Serialized::Continue) => {
          tup!($it, $serializers) = Some(ser);
          return Ok(($index + i, Serialized::Continue))
        },
        (i, Serialized::Done) => {
          $index += i;
        }
      }
    }

    return Ok(($index, Serialized::Done));
  };
);

#[macro_export]
macro_rules! seq (
  ($($elem:expr),*, $output: ident) => (
    {
      use ::std::result::Result::*;
      use ::std::option::Option::*;

      let mut res = sequence_init!((), $($elem:expr),*);

      let mut i = 0;
      seq_ser!(0, res, i, $output, $($elem:expr),*)
    }
  );
);

#[doc(hidden)]
#[macro_export]
macro_rules! sequence_init (
  ((), $e:expr, $($rest:tt)*) => (
    sequence_init!((Some($e)), $($rest)*)
  );

  (($($parsed:expr),*), $e:expr, $($rest:tt)*) => (
    sequence_init!(($($parsed),* , Some($e)), $($rest)*);
  );

  (($($parsed:expr),*), $e:expr) => (
    ($($parsed),* , Some($e))
  );
  (($($parsed:expr),*),) => (
    ($($parsed),*)
  );
);

#[doc(hidden)]
#[macro_export]
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

// HACK: for some reason, Rust 1.11 does not accept $res.$it in
// permutation_unwrap. This is a bit ugly, but it will have no
// impact on the generated code
#[doc(hidden)]
#[macro_export]
macro_rules! tup (
  (0, $tup:expr) => ($tup.0);
  (1, $tup:expr) => ($tup.1);
  (2, $tup:expr) => ($tup.2);
  (3, $tup:expr) => ($tup.3);
  (4, $tup:expr) => ($tup.4);
  (5, $tup:expr) => ($tup.5);
  (6, $tup:expr) => ($tup.6);
  (7, $tup:expr) => ($tup.7);
  (8, $tup:expr) => ($tup.8);
  (9, $tup:expr) => ($tup.9);
  (10, $tup:expr) => ($tup.10);
  (11, $tup:expr) => ($tup.11);
  (12, $tup:expr) => ($tup.12);
  (13, $tup:expr) => ($tup.13);
  (14, $tup:expr) => ($tup.14);
  (15, $tup:expr) => ($tup.15);
  (16, $tup:expr) => ($tup.16);
  (17, $tup:expr) => ($tup.17);
  (18, $tup:expr) => ($tup.18);
);

pub struct Then3<A, B, C> {
  pub serializers: (Option<A>, Option<B>, Option<C>),
}

impl<A: Serializer, B: Serializer, C: Serializer> Serializer for Then3<A, B, C> {
  #[inline(always)]
  fn serialize<'b, 'c>(&'c mut self, output: &'b mut [u8]) -> Result<(usize, Serialized), GenError> {
    let mut i = 0;
    seq_ser!(0, self.serializers, i, output, 1, 2, 3);
  }
}

pub struct Then4<A, B, C, D> {
  pub serializers: (Option<A>, Option<B>, Option<C>, Option<D>),
}

impl<A: Serializer, B: Serializer, C: Serializer, D: Serializer> Serializer for Then4<A, B, C, D> {
  #[inline(always)]
  fn serialize<'b, 'c>(&'c mut self, output: &'b mut [u8]) -> Result<(usize, Serialized), GenError> {
    let mut i = 0;
    seq_ser!(0, self.serializers, i, output, 1, 2, 3, 4);
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
  fn serialize<'b, 'c>(&'c mut self, output: &'b mut [u8]) -> Result<(usize, Serialized), GenError> {
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
  fn serialize<'b, 'c>(&'c mut self, output: &'b mut [u8]) -> Result<(usize, Serialized), GenError> {
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
  fn raw<'a>(&'a self) -> SliceSerializer<'a>;
}

impl<S: AsRef<str>> StrSr for S {
  #[inline(always)]
  fn raw<'a>(&'a self) -> SliceSerializer<'a> {
    SliceSerializer::new(self.as_ref().as_bytes())
  }
}

#[test]
fn str_serializer() {
  let s = String::from("hello world!");
  let mut sr = SliceSerializer::new(s.as_str().as_bytes());

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
  let sr1 = SliceSerializer::new(s1.as_str().as_bytes());

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

#[derive(Clone, Debug, PartialEq)]
pub enum JsonValue {
  Str(String),
  Boolean(bool),
  Num(f64),
  Array(Vec<JsonValue>),
  Object(HashMap<String, JsonValue>),
}

pub fn gen_json_value<'a>(g: &'a JsonValue) -> Box<Serializer + 'a> {
  match g {
    JsonValue::Str(ref s) => Box::new(gen_str(s)) as Box<Serializer>,
    JsonValue::Boolean(ref b) => Box::new(gen_bool(b)) as Box<Serializer>,
    JsonValue::Num(ref n) => Box::new(gen_num(n)) as Box<Serializer>,
    JsonValue::Array(ref v) => Box::new(gen_array(v)) as Box<Serializer>,
    JsonValue::Object(ref o) => Box::new(gen_object(o)) as Box<Serializer>,
  }
}

#[derive(Clone, Debug, PartialEq)]
pub enum JsonValueSerializer {
  Str(String),
  Boolean(bool),
  Num(f64),
  Array(Vec<JsonValue>),
  Object(HashMap<String, JsonValue>),
}

#[inline(always)]
pub fn gen_str<'a, S: AsRef<str>>(s: &'a S) -> impl Serializer + 'a {
  "\"".raw()
    .then(s.raw())
    .then("\"".raw())
}

pub fn gen_bool(b: &bool) -> impl Serializer {
  if *b {
    "true".raw()
  } else {
    "false".raw()
  }
}

pub fn gen_num(_b: &f64) -> impl Serializer {
  "1234.56".raw()
}

pub fn gen_array<'a>(arr: &'a [JsonValue]) -> impl Serializer + 'a {
  let sr = "[".raw();

  sr.then(or(
      if arr.len() > 0 {
        Some(gen_json_value(&arr[0]))
      } else {
        None
      },
      empty()
      )).then(All::new(arr.iter().skip(1).map(|v| {
    ",".raw().then(gen_json_value(v))
  })))
  .then("]".raw())
}

pub fn gen_object<'a>(o: &'a HashMap<String, JsonValue>) -> impl Serializer + 'a {
  let sr = "{".raw();
  let len = o.len();

  let mut iter = o.iter();
  sr.then(or(
      if len > 0 {
        let first = iter.next().unwrap();
        Some(gen_key_value(first))
      } else {
        None
      },
      empty()
  )).then(All::new(iter.map(|v| {
    ",".raw().then(gen_key_value(v))
  })))
  .then("}".raw())
}

pub fn gen_key_value<'a>(kv: (&'a String, &'a JsonValue)) -> impl Serializer + 'a {
  gen_str(kv.0).then(":".raw()).then(gen_json_value(&kv.1))
}

// from https://github.com/bluss/maplit
macro_rules! hashmap {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(hashmap!(@single $rest)),*]));

    ($($key:expr => $value:expr,)+) => { hashmap!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let _cap = hashmap!(@count $($key),*);
            let mut _map = ::std::collections::HashMap::with_capacity(_cap);
            $(
                let _ = _map.insert($key, $value);
            )*
            _map
        }
    };
}

#[test]
fn json_test() {
  use std::iter::repeat;
  let value = JsonValue::Object(hashmap!{
    String::from("arr") => JsonValue::Array(vec![JsonValue::Num(1.0), JsonValue::Num(12.3), JsonValue::Num(42.0)]),
    String::from("b") => JsonValue::Boolean(true),
    String::from("o") => JsonValue::Object(hashmap!{
      String::from("x") => JsonValue::Str(String::from("abcd")),
      String::from("y") => JsonValue::Str(String::from("efgh")),
      String::from("empty") => JsonValue::Array(vec![]),
    }),
  });

  //let value = JsonValue::Array(repeat(element).take(10).collect::<Vec<JsonValue>>());
  let mut sr = gen_json_value(&value);

  let mut buffer = repeat(0).take(16384).collect::<Vec<u8>>();
  let (index, result) = sr.serialize(&mut buffer).unwrap();

  println!("result:\n{}", str::from_utf8(&buffer[..index]).unwrap());
  assert_eq!(str::from_utf8(&buffer[..index]).unwrap(),
    "{\"arr\":[1234.56,1234.56,1234.56],\"b\":true,\"o\":{\"empty\":[],\"x\":\"abcd\",\"y\":\"efgh\"}}");
  //panic!();
}


