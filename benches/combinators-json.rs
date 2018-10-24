#![feature(test)]
extern crate test;
#[macro_use]
extern crate cookie_factory;
#[macro_use]
extern crate maplit;

use std::str;
use std::iter::repeat;
use std::collections::BTreeMap;

use cookie_factory::*;
use test::Bencher;

#[derive(Clone, Debug, PartialEq)]
pub enum JsonValue {
  Str(String),
  Boolean(bool),
  Num(f64),
  Array(Vec<JsonValue>),
  Object(BTreeMap<String, JsonValue>),
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

pub fn gen_object<'a>(o: &'a BTreeMap<String, JsonValue>) -> impl Serializer + 'a {
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

#[test]
fn json_test() {
  use std::iter::repeat;
  let value = JsonValue::Object(btreemap!{
    String::from("arr") => JsonValue::Array(vec![JsonValue::Num(1.0), JsonValue::Num(12.3), JsonValue::Num(42.0)]),
    String::from("b") => JsonValue::Boolean(true),
    String::from("o") => JsonValue::Object(btreemap!{
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

#[bench]
fn combinators_json(b: &mut Bencher) {
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

  let mut buffer = repeat(0).take(16384).collect::<Vec<u8>>();
  let index = {
    let mut sr = gen_json_value(&value);
    let (index, result) = sr.serialize(&mut buffer).unwrap();

    println!("result:\n{}", str::from_utf8(&buffer[..index]).unwrap());
    //panic!();

    index as u64
  };

  b.bytes = index;
  b.iter(|| {
    let mut sr = gen_json_value(&value);
    let (index, result) = sr.serialize(&mut buffer).unwrap();
    index
  });
}

#[bench]
fn combinators_json_create_serializer(b: &mut Bencher) {
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

  b.iter(|| {
    gen_json_value(&value)
  });
}

#[bench]
fn combinators_gen_str_create_serializer(b: &mut Bencher) {
  let mut buffer = repeat(0).take(16384).collect::<Vec<u8>>();
  let index = {
    let mut sr = gen_str(&"hello");
    let (index, result) = sr.serialize(&mut buffer).unwrap();

    println!("result:\n{}", str::from_utf8(&buffer[..index]).unwrap());
    //panic!();

    index as u64
  };

  b.bytes = index;
  b.iter(|| {
    gen_str(&"hello")
  });
}

#[bench]
fn combinators_gen_str(b: &mut Bencher) {
  let mut buffer = repeat(0).take(16384).collect::<Vec<u8>>();
  let index = {
    let mut sr = gen_str(&"hello");
    let (index, result) = sr.serialize(&mut buffer).unwrap();

    println!("result:\n{}", str::from_utf8(&buffer[..index]).unwrap());
    //panic!();

    index as u64
  };

  b.bytes = index;
  b.iter(|| {
    let mut sr = gen_str(&"hello");
    let (index, result) = sr.serialize(&mut buffer).unwrap();
    index
  });
}
