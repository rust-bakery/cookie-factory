use std::str;
use std::collections::BTreeMap;

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
    let (out, len1) = string("\"")(out)?;
    let (out, len2) = string(s)(out)?;
    string("\"")(out).map(|(out, len)| (out, len + len1 + len2))
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
pub fn gen_num<'a>(_b: f64) -> impl SerializeFn<&'a mut [u8]> {
  string("1234.56")
}

pub fn gen_array<'a, 'b: 'a>(arr: &'b [JsonValue]) -> impl SerializeFn<&'a mut [u8]> {
  move |out: &'a mut [u8]| {
    let (out, len1) = string("[")(out)?;
    let (out, len2) = separated_list(string(","), arr.iter().map(gen_json_value))(out)?;
    string("]")(out).map(|(out, len)| (out, len + len1 + len2))
  }
}

pub fn gen_key_value<'a, 'b: 'a>(kv: (&'b String, &'b JsonValue)) -> impl SerializeFn<&'a mut [u8]> {
  move |out: &'a mut [u8]| {
    let (out, len1) = gen_str(kv.0)(out)?;
    let (out, len2) = string(":")(out)?;
    gen_json_value(&kv.1)(out).map(|(out, len)| (out, len + len1 + len2))
  }
}

pub fn gen_object<'a, 'b: 'a>(o: &'b BTreeMap<String, JsonValue>) -> impl SerializeFn<&'a mut [u8]> {
  move |out: &'a mut [u8]| {
    let (out, len1) = string("{")(out)?;

    let (out, len2) = separated_list(string(","), o.iter().map(gen_key_value))(out)?;
    string("}")(out).map(|(out, len)| (out, len + len1 + len2))
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

  let mut buffer = repeat(0).take(16384).collect::<Vec<u8>>();
  let pos = {
    let mut sr = gen_json_value(&value);

    let (res, _) = sr(&mut buffer).unwrap();
    res.as_ptr() as usize
  };

  let index = pos - buffer.as_ptr() as usize;


  println!("result:\n{}", str::from_utf8(&buffer[..index]).unwrap());
  assert_eq!(str::from_utf8(&buffer[..index]).unwrap(),
    "{\"arr\":[1234.56,1234.56,1234.56],\"b\":true,\"o\":{\"empty\":[],\"x\":\"abcd\",\"y\":\"efgh\"}}");
}
