//#![feature(trace_macros)]
#![feature(test)]
extern crate test;
#[macro_use]
extern crate cookie_factory;

use std::str;
use std::iter::repeat;
use std::collections::HashMap;

use cookie_factory::*;
use test::Bencher;

#[derive(Clone, Debug, PartialEq)]
pub enum JsonValue {
  Str(String),
  Boolean(bool),
  Num(f64),
  Array(Vec<JsonValue>),
  Object(HashMap<String, JsonValue>),
}

pub fn gen_json_value<'a>(x:(&'a mut [u8],usize), g:&JsonValue) -> Result<(&'a mut [u8],usize),GenError> {
  match g {
    JsonValue::Str(ref s) => gen_str(x, s),
    JsonValue::Boolean(b) => gen_bool(x, b),
    JsonValue::Num(ref f) => gen_num(x, f),
    JsonValue::Array(ref v) => gen_array(x, v),
    JsonValue::Object(ref o) => gen_object(x, o),
  }
}

pub fn gen_str<'a>(x:(&'a mut [u8],usize), s:&String) -> Result<(&'a mut [u8],usize),GenError> {
  do_gen!(x,
    gen_slice!(&b"\""[..]) >>
    gen_slice!(s.as_bytes()) >>
    gen_slice!(&b"\""[..])
  )
}

pub fn gen_bool<'a>(x:(&'a mut [u8],usize), b:&bool) -> Result<(&'a mut [u8],usize),GenError> {
  let sl = match b {
    true => &b"true"[..],
    false => &b"false"[..],
  };

  gen_slice!(x, sl)
}

pub fn gen_num<'a>(x:(&'a mut [u8],usize), b:&f64) -> Result<(&'a mut [u8],usize),GenError> {
  //TODO
  gen_slice!(x, &b"1234.56"[..])
}

pub fn gen_array<'a>(x:(&'a mut [u8],usize), arr:&[JsonValue]) -> Result<(&'a mut [u8],usize),GenError> {
  let mut output = gen_slice!(x, &b"["[..])?;
  if arr.len() > 0 {
    output = gen_json_value(output, &arr[0])?;

    if arr.len() > 1 {
      output = gen_many!((output.0, output.1),
        &arr[1..],
        gen_array_element
      )?;
    }
  }
  gen_slice!(output, &b"]"[..])
}

pub fn gen_array_element<'a>(x:(&'a mut [u8],usize), val: &JsonValue) -> Result<(&'a mut [u8],usize),GenError> {
  do_gen!(x,
    gen_slice!(&b","[..]) >>
    gen_call!(gen_json_value, val)
  )
}

pub fn gen_object<'a>(x:(&'a mut [u8],usize), o:&HashMap<String, JsonValue>) -> Result<(&'a mut [u8],usize),GenError> {
  let mut output = gen_slice!(x, &b"{"[..])?;
  let mut it = o.iter();

  if let Some((key, value)) = it.next() {
    output = gen_object_element(output, key, value, true)?;
  }

  for (key, value) in it {
    output = gen_object_element(output, key, value, false)?;
  }

  gen_slice!(output, &b"]"[..])
}

pub fn gen_object_element<'a>(x:(&'a mut [u8],usize), key: &String, value:&JsonValue, is_first: bool) -> Result<(&'a mut [u8],usize),GenError> {
  let mut output = x;
  if !is_first {
    output = gen_slice!(output, &b","[..])?;
  }

  do_gen!(output,
    gen_call!(gen_str, key) >>
    gen_slice!(&b":"[..]) >>
    gen_call!(gen_json_value, value)
  )
}
/*
named!(float<f32>, flat_map!(recognize_float, parse_to!(f32)));

//FIXME: verify how json strings are formatted
named!(
  string<&str>,
  delimited!(
    char!('\"'),
    map_res!(
      escaped!(call!(alphanumeric), '\\', one_of!("\"n\\")),
      str::from_utf8
    ),
    //map_res!(escaped!(take_while1!(is_alphanumeric), '\\', one_of!("\"n\\")), str::from_utf8),
    char!('\"')
  )
);

named!(
  boolean<bool>,
  alt!(value!(false, tag!("false")) | value!(true, tag!("true")))
);

named!(
  array<Vec<JsonValue>>,
  ws!(delimited!(
    char!('['),
    separated_list!(char!(','), value),
    char!(']')
  ))
);

named!(
  key_value<(&str, JsonValue)>,
  ws!(separated_pair!(string, char!(':'), value))
);

named!(
  hash<HashMap<String, JsonValue>>,
  ws!(map!(
    delimited!(
      char!('{'),
      separated_list!(char!(','), key_value),
      char!('}')
    ),
    |tuple_vec| tuple_vec
      .into_iter()
      .map(|(k, v)| (String::from(k), v))
      .collect()
  ))
);

named!(
  value<JsonValue>,
  ws!(alt!(
      hash    => { |h| JsonValue::Object(h)            } |
      array   => { |v| JsonValue::Array(v)             } |
      string  => { |s| JsonValue::Str(String::from(s)) } |
      float   => { |f| JsonValue::Num(f)               } |
      boolean => { |b| JsonValue::Boolean(b)           }
    ))
);
*/

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

#[bench]
fn json_bench(b: &mut Bencher) {
  let element = JsonValue::Object(hashmap!{
    String::from("arr") => JsonValue::Array(vec![JsonValue::Num(1.0), JsonValue::Num(12.3), JsonValue::Num(42.0)]),
    String::from("b") => JsonValue::Boolean(true),
    String::from("o") => JsonValue::Object(hashmap!{
      String::from("x") => JsonValue::Str(String::from("abcd")),
      String::from("y") => JsonValue::Str(String::from("efgh")),
      String::from("empty") => JsonValue::Array(vec![]),
    }),
  });

  let value = JsonValue::Array(repeat(element).take(10).collect::<Vec<JsonValue>>());

  let mut buffer = repeat(0).take(16384).collect::<Vec<u8>>();
  let ptr = {
    let (buf, index) = gen_json_value((&mut buffer, 0), &value).unwrap();

    println!("result:\n{}", str::from_utf8(buf).unwrap());
    //panic!();

    buf.as_ptr() as u64
  };

  b.bytes = ptr - (&buffer).as_ptr() as u64;
  b.iter(|| {
    let res = gen_json_value((&mut buffer, 0), &value).unwrap();
    res.1
  });
}

