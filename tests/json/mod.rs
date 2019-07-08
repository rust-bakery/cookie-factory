use std::str;
use std::collections::BTreeMap;

use cookie_factory::*;
use cookie_factory::lib::std::io::Write;

#[derive(Clone, Debug, PartialEq)]
pub enum JsonValue {
  Str(String),
  Boolean(bool),
  Num(f64),
  Array(Vec<JsonValue>),
  Object(BTreeMap<String, JsonValue>),
}

#[inline(always)]
pub fn gen_str<'a, 'b: 'a, W: Write + 'a>(s: &'b str) -> impl SerializeFn<W> + 'a {
   tuple((
     string("\""),
     string(s),
     string("\""),
   ))
}

#[inline(always)]
pub fn gen_bool<W: Write>(b: bool) -> impl SerializeFn<W> {
  if b {
    string("true")
  } else {
    string("false")
  }
}

#[inline(always)]
pub fn gen_num<W: Write>(_b: f64) -> impl SerializeFn<W> {
  string("1234.56")
}

pub fn gen_array<'a, 'b: 'a, W: Write + 'a>(arr: &'b [JsonValue]) -> impl SerializeFn<W> + 'a {
  tuple((
    string("["),
    separated_list(string(","), arr.iter().map(gen_json_value)),
    string("]")
  ))
}

pub fn gen_key_value<'a, 'b: 'a, W: Write + 'a>(kv: (&'b String, &'b JsonValue)) -> impl SerializeFn<W> + 'a {
  tuple((
    gen_str(kv.0),
    string(":"),
    gen_json_value(&kv.1),
  ))
}

pub fn gen_object<'a, 'b: 'a, W: Write + 'a>(o: &'b BTreeMap<String, JsonValue>) -> impl SerializeFn<W> + 'a {
  tuple((
    string("{"),
    separated_list(string(","), o.iter().map(gen_key_value)),
    string("}"),
  ))
}

pub fn gen_json_value<'a, W: Write>(g: &'a JsonValue) -> impl SerializeFn<W> + 'a {
  move |out: W| {
    match g {
      JsonValue::Str(ref s) => gen_str(s)(out),
      JsonValue::Boolean(ref b) => gen_bool(*b)(out),
      JsonValue::Num(ref n) => gen_num(*n)(out),
      JsonValue::Array(ref v) => gen_array(v)(out),
      JsonValue::Object(ref o) => gen_object(o)(out),
    }
  }
}

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
