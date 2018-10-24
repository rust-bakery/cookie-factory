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
  "[".raw()
    .then(SeparatedList::new(
        ",".raw(),
        arr.iter().map(gen_json_value)))
    .then("]".raw())
}

pub fn gen_object<'a>(o: &'a BTreeMap<String, JsonValue>) -> impl Serializer + 'a {
  "{".raw()
    .then(SeparatedList::new(
        ",".raw(),
        o.iter().map(gen_key_value)))
    .then("}".raw())
}

pub fn gen_key_value<'a>(kv: (&'a String, &'a JsonValue)) -> impl Serializer + 'a {
  gen_str(kv.0).then(":".raw()).then(gen_json_value(&kv.1))
}

