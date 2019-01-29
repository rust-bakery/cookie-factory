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

/*
pub fn gen_json_value<'a>(g: &'a JsonValue) -> Box<Serializer + 'a> {
  match g {
    JsonValue::Str(ref s) => Box::new(gen_str(s)) as Box<Serializer>,
    JsonValue::Boolean(ref b) => Box::new(gen_bool(b)) as Box<Serializer>,
    JsonValue::Num(ref n) => Box::new(gen_num(n)) as Box<Serializer>,
    JsonValue::Array(ref v) => Box::new(gen_array(v)) as Box<Serializer>,
    JsonValue::Object(ref o) => Box::new(gen_object(o)) as Box<Serializer>,
  }
}
*/

pub fn gen_json_value<'a>(g: &'a JsonValue) -> impl Serializer + 'a {
  or(
    if let JsonValue::Str(ref s) = g {
      Some(gen_str(s))
    } else {
      None
    },
    or(
      if let JsonValue::Boolean(ref b) = g {
        Some(gen_bool(b))
      } else {
        None
      },
      or(
        if let JsonValue::Num(ref n) = g {
          Some(gen_num(n))
        } else {
          None
        },
        or(
          if let JsonValue::Array(ref v) = g {
            Some(ArrayWrap::new(v))
          } else {
            None
          },
          or(
            if let JsonValue::Object(ref o) = g {
              Some(ObjectWrap::new(o))
            } else {
              None
            },
            empty()
          )
        )
      )
    )
  )
}

pub struct ArrayWrap<'a> {
  pub value: &'a Vec<JsonValue>,
  pub serializer: Option<Box<Serializer + 'a>>,
}

impl<'a> ArrayWrap<'a> {
  pub fn new(value: &'a Vec<JsonValue>) -> ArrayWrap<'a> {
    ArrayWrap {
      value,
      serializer: None,
    }
  }
}

impl<'a> Serializer for ArrayWrap<'a> {
  #[inline(always)]
  fn serialize<'b, 'c>(&'b mut self, output: &'c mut [u8]) -> Result<(usize, Serialized), GenError> {
    if let Some(s) = self.serializer.as_mut() {
      return s.serialize(output);
    }

    let mut s = gen_array(self.value);
    match s.serialize(output)? {
      (i, Serialized::Continue) => {
        self.serializer = Some(Box::new(s) as Box<Serializer>);
        Ok((i, Serialized::Continue))
      },
      (i, Serialized::Done) => Ok((i, Serialized::Done)),
    }
  }
}

pub struct ObjectWrap<'a> {
  pub value: &'a BTreeMap<String, JsonValue>,
  pub serializer: Option<Box<Serializer + 'a>>,
}

impl<'a> ObjectWrap<'a> {
  pub fn new(value: &'a BTreeMap<String, JsonValue>) -> ObjectWrap<'a> {
    ObjectWrap {
      value,
      serializer: None,
    }
  }
}

impl<'a> Serializer for ObjectWrap<'a> {
  #[inline(always)]
  fn serialize<'b, 'c>(&'b mut self, output: &'c mut [u8]) -> Result<(usize, Serialized), GenError> {
    if let Some(s) = self.serializer.as_mut() {
      return s.serialize(output);
    }

    let mut s = gen_object(self.value);
    match s.serialize(output)? {
      (i, Serialized::Continue) => {
        self.serializer = Some(Box::new(s) as Box<Serializer>);
        Ok((i, Serialized::Continue))
      },
      (i, Serialized::Done) => Ok((i, Serialized::Done)),
    }
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

