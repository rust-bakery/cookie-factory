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

#[path="../tests/json/mod.rs"] mod implementation;
use implementation::*;

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
