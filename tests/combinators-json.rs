#![feature(test)]
extern crate test;
#[macro_use]
extern crate cookie_factory;
#[macro_use]
extern crate maplit;

use cookie_factory::*;

#[path="./json/mod.rs"] mod implementation;
use implementation::*;

#[test]
fn json_test() {
  use std::str;
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

  let mut buffer = repeat(0).take(16384).collect::<Vec<u8>>();
  let ptr = {
    let mut sr = gen_json_value(&value);
    let _res = sr(&mut buffer).unwrap();
    _res.as_ptr() as usize
  };

  let index = ptr - (&buffer[..]).as_ptr() as usize;

  println!("result:\n{}", str::from_utf8(&buffer[..index]).unwrap());
  assert_eq!(str::from_utf8(&buffer[..index]).unwrap(),
    "{\"arr\":[1234.56,1234.56,1234.56],\"b\":true,\"o\":{\"empty\":[],\"x\":\"abcd\",\"y\":\"efgh\"}}");
}

