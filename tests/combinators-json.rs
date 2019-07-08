extern crate cookie_factory;
#[macro_use]
extern crate maplit;

#[path="./json/mod.rs"] mod implementation;
use crate::implementation::*;

#[test]
fn json_test() {
  use std::str;
  use cookie_factory::lib::std::io::Cursor;

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

  let mut buffer = [0u8; 8192];
  let sr = gen_json_value(&value);
  let writer = Cursor::new(&mut buffer[..]);
  let writer = sr(writer).unwrap();
  let size = writer.position() as usize;
  let buffer = writer.into_inner();

  println!("result:\n{}", str::from_utf8(&buffer[..size]).unwrap());
  assert_eq!(str::from_utf8(&buffer[..size]).unwrap(),
    "{\"arr\":[1234.56,1234.56,1234.56],\"b\":true,\"o\":{\"empty\":[],\"x\":\"abcd\",\"y\":\"efgh\"}}");
}

