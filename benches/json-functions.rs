#![feature(test)]
extern crate test;
#[macro_use]
extern crate cookie_factory;
#[macro_use]
extern crate maplit;

use std::str;
use std::collections::BTreeMap;
use test::Bencher;

use cookie_factory::*;

#[derive(Clone, Debug, PartialEq)]
pub enum JsonValue {
  Str(String),
  Boolean(bool),
  Num(f64),
  Array(Vec<JsonValue>),
  Object(BTreeMap<String, JsonValue>),
}

#[inline(always)]
pub fn gen_str<'a, 'b: 'a>(s: &'b str) -> impl SerializeFn<&'a mut [u8]> {
  move |out: &'a mut [u8]| {
    let out = string("\"")(out)?;
    let out = string(s)(out)?;
    string("\"")(out)
  }
}

#[inline(always)]
pub fn gen_bool<'a>(b: bool) -> impl SerializeFn<&'a mut [u8]> {
  if b {
    string("true")
  } else {
    string("false")
  }
}

#[inline(always)]
pub fn gen_num<'a>(b: f64) -> impl SerializeFn<&'a mut [u8]> {
  /*move |out: &'a mut [u8]| {
    let s = format!("{}", b);
    string(s)(out)
  }*/
  string("1234.56")
}

pub fn gen_array<'a, 'b: 'a>(arr: &'b [JsonValue]) -> impl SerializeFn<&'a mut [u8]> {
  move |out: &'a mut [u8]| {
    let out = string("[")(out)?;
    let out = separated_list(string(","), arr.iter().map(gen_json_value))(out)?;
    string("]")(out)
  }
}

pub fn gen_key_value<'a, 'b: 'a>(kv: (&'b String, &'b JsonValue)) -> impl SerializeFn<&'a mut [u8]> {
  move |out: &'a mut [u8]| {
    let out = gen_str(kv.0)(out)?;
    let out = string(":")(out)?;
    gen_json_value(&kv.1)(out)
  }
}

pub fn gen_object<'a, 'b: 'a>(o: &'b BTreeMap<String, JsonValue>) -> impl SerializeFn<&'a mut [u8]> {
  move |out: &'a mut [u8]| {
    let out = string("{")(out)?;

    let out = separated_list(string(","), o.iter().map(gen_key_value))(out)?;
    string("}")(out)
  }
}


pub fn gen_json_value<'a>(g: &'a JsonValue) -> impl SerializeFn<&'a mut [u8]> {
  move |out: &'a mut [u8]| {
    match g {
      JsonValue::Str(ref s) => gen_str(s)(out),
      JsonValue::Boolean(ref b) => gen_bool(*b)(out),
      JsonValue::Num(ref n) => gen_num(*n)(out),
      JsonValue::Array(ref v) => gen_array(v)(out),
      JsonValue::Object(ref o) => gen_object(o)(out),
    }
  }
}

use std::iter::repeat;

#[test]
fn json_test() {
  let value = JsonValue::Object(btreemap!{
    String::from("arr") => JsonValue::Array(vec![JsonValue::Num(1.0), JsonValue::Num(12.3), JsonValue::Num(42.0)]),
    String::from("b") => JsonValue::Boolean(true),
    String::from("o") => JsonValue::Object(btreemap!{
      String::from("x") => JsonValue::Str(String::from("abcd")),
      String::from("y") => JsonValue::Str(String::from("efgh")),
      String::from("empty") => JsonValue::Array(vec![]),
    }),
  });

  let mut buffer = repeat(0).take(16384).collect::<Vec<u8>>();
  let pos = {
    let sr = gen_json_value(&value);

    let res = sr(&mut buffer[..]).unwrap();
    res.as_ptr() as usize
  };

  let index = pos - buffer.as_ptr() as usize;


  println!("result:\n{}", str::from_utf8(&buffer[..index]).unwrap());
  assert_eq!(str::from_utf8(&buffer[..index]).unwrap(),
    "{\"arr\":[1234.56,1234.56,1234.56],\"b\":true,\"o\":{\"empty\":[],\"x\":\"abcd\",\"y\":\"efgh\"}}");
  panic!();
}

#[bench]
fn functions_json(b: &mut Bencher) {
  let element = JsonValue::Object(btreemap!{
    String::from("arr") => JsonValue::Array(vec![JsonValue::Num(1.0), JsonValue::Num(12.3), JsonValue::Num(42.0)]),
    String::from("b") => JsonValue::Boolean(true),
    String::from("o") => JsonValue::Object(btreemap!{
      String::from("x") => JsonValue::Str(String::from("abcd")),
      String::from("y") => JsonValue::Str(String::from("efgh")),
      String::from("empty") => JsonValue::Array(vec![]),
    }),
  });

  let value = JsonValue::Array(repeat(element).take(10).collect::<Vec<JsonValue>>());

  let mut buffer = repeat(0u8).take(16384).collect::<Vec<_>>();
  let pos = {
    let sr = gen_json_value(&value);

    let res = sr(&mut buffer[..]).unwrap();
    res.as_ptr() as usize
  };

  let index = pos - buffer.as_ptr() as usize;

  b.bytes = index as u64;
  b.iter(|| {
    let sr = gen_json_value(&value);
    let _ = sr(&mut buffer[..]).unwrap();
  });
}

#[bench]
fn functions_gen_str(b: &mut Bencher) {
  let mut buffer = repeat(0).take(16384).collect::<Vec<u8>>();

  let pos = {
    let sr = gen_str(&"hello");

    let res = sr(&mut buffer[..]).unwrap();
    res.as_ptr() as usize
  };

  let index = pos - buffer.as_ptr() as usize;

  b.bytes = index as u64;
  b.iter(|| {
    let sr = gen_str(&"hello");
    let _ = sr(&mut buffer[..]).unwrap();
  });
}
