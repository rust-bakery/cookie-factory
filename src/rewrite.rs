
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
  fn serialize<'b, 'c>(&'c mut self, _output: &'b mut [u8]) -> Result<(usize, Serialized), GenError> {
    Ok((0, Serialized::Done))
  }
}

pub fn empty() -> EmptySerializer {
  EmptySerializer
}

#[derive(Debug)]
pub struct StrSerializer<'a> {
  value: &'a str,
  index: usize,
}

impl<'a> StrSerializer<'a> {
  pub fn new(s: &'a str) -> StrSerializer<'a> {
    StrSerializer {
      value: s,
      index: 0,
    }
  }
}

//use std::cmp::min;
impl<'a> Serializer for StrSerializer<'a> {
  fn serialize<'b, 'c>(&'c mut self, output: &'b mut [u8]) -> Result<(usize, Serialized), GenError> {
    let output_len = output.len();
    let self_len = (&self.value.as_bytes()[self.index..]).len();
    if self_len > output_len {
      output.clone_from_slice(&self.value.as_bytes()[self.index..self.index+output_len]);
      self.index += output_len;
      Ok((output_len, Serialized::Continue))
    } else {
      (&mut output[..self_len]).clone_from_slice(&self.value.as_bytes()[self.index..]);
      self.index = self.value.as_bytes().len();
      Ok((self_len, Serialized::Done))
    }
  }
}

impl<S: ?Sized + Serializer> Serializer for Box<S> {
  fn serialize<'b, 'c>(&'c mut self, output: &'b mut [u8]) -> Result<(usize, Serialized), GenError> {
    (**self).serialize(output)
  }
}

pub struct Then<A, B> {
  a: A,
  b: B,
  first_done: bool,
}

impl<A:Serializer, B:Serializer> Then<A, B> {
  pub fn new(a: A, b: B) -> Self {
    Then {
      a,
      b,
      first_done: false,
    }
  }
}

impl<A:Serializer, B:Serializer> Serializer for Then<A,B> {
  fn serialize<'b, 'c>(&'c mut self, output: &'b mut [u8]) -> Result<(usize, Serialized), GenError> {
    let mut i = 0;
    if !self.first_done {
      match self.a.serialize(output)? {
        (index, Serialized::Continue) => return Ok((index, Serialized::Continue)),
        (index, Serialized::Done) => {
          self.first_done = true;
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
  pub fn new(a: Option<A>, b: B) -> Self {
    Or {
      a,
      b,
    }
  }
}

impl<A:Serializer, B:Serializer> Serializer for Or<A,B> {
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
      if self.current.is_none() {
        self.current = self.it.next();
        if self.current.is_none() {
          return Ok((index, Serialized::Done));
        }
      }

      assert!(index <= output.len());
      let sl = &mut output[index..];
      match self.current.as_mut().unwrap().serialize(sl)? {
        (i, Serialized::Continue) => return Ok((index + i, Serialized::Continue)),
        (i, Serialized::Done) => {
          index += i;
          self.current = None;
        },
      }
    }
  }
}


pub trait StrSr {
  fn raw<'a>(&'a self) -> StrSerializer<'a>;
}

impl<S: AsRef<str>> StrSr for S {
  fn raw<'a>(&'a self) -> StrSerializer<'a> {
    StrSerializer::new(self.as_ref())
  }
}

#[test]
fn str_serializer() {
  let s = String::from("hello world!");
  let mut sr = StrSerializer::new(s.as_str());

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
  let sr1 = StrSerializer::new(s1.as_str());

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
    "{\"arr\":[1234.56,1234.56,1234.56,1234.56],\"b\":true,\"o\":{\"empty\":[],\"x\":\"abcd\",\"y\":\"efgh\"}}");
  //panic!();
}


