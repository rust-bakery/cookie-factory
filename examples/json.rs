extern crate cookie_factory;
#[macro_use]
extern crate maplit;

use std::{str, iter::repeat};

#[path="../tests/json/mod.rs"] mod implementation;
use implementation::*;

fn main() {
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

  let size = {
    let sr = gen_json_value(&value);
    let (_, size) = sr(&mut buffer).unwrap();
    size
  };

  println!("result:\n{}", str::from_utf8(&buffer[..size]).unwrap());
}

