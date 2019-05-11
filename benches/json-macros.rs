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

pub fn gen_json_value<'a>(x:(&'a mut [u8],usize), g:&JsonValue) -> Result<(&'a mut [u8],usize),GenError> {
  match g {
    JsonValue::Str(ref s) => gen_str(x, s),
    JsonValue::Boolean(b) => gen_bool(x, b),
    JsonValue::Num(ref f) => gen_num(x, f),
    JsonValue::Array(ref v) => gen_array(x, v),
    JsonValue::Object(ref o) => gen_object(x, o),
  }
}

pub fn gen_str<'a>(x:(&'a mut [u8],usize), s:&String) -> Result<(&'a mut [u8],usize),GenError> {
  do_gen!(x,
    gen_slice!(&b"\""[..]) >>
    gen_slice!(s.as_bytes()) >>
    gen_slice!(&b"\""[..])
  )
}

pub fn gen_bool<'a>(x:(&'a mut [u8],usize), b:&bool) -> Result<(&'a mut [u8],usize),GenError> {
  let sl = match b {
    true => &b"true"[..],
    false => &b"false"[..],
  };

  gen_slice!(x, sl)
}

pub fn gen_num<'a>(x:(&'a mut [u8],usize), b:&f64) -> Result<(&'a mut [u8],usize),GenError> {
  //TODO
  gen_slice!(x, &b"1234.56"[..])
}

pub fn gen_array<'a>(x:(&'a mut [u8],usize), arr:&[JsonValue]) -> Result<(&'a mut [u8],usize),GenError> {
  let mut output = gen_slice!(x, &b"["[..])?;
  if arr.len() > 0 {
    output = gen_json_value(output, &arr[0])?;

    if arr.len() > 1 {
      output = gen_many!((output.0, output.1),
        &arr[1..],
        gen_array_element
      )?;
    }
  }
  gen_slice!(output, &b"]"[..])
}

pub fn gen_array_element<'a>(x:(&'a mut [u8],usize), val: &JsonValue) -> Result<(&'a mut [u8],usize),GenError> {
  do_gen!(x,
    gen_slice!(&b","[..]) >>
    gen_call!(gen_json_value, val)
  )
}

pub fn gen_object<'a>(x:(&'a mut [u8],usize), o:&BTreeMap<String, JsonValue>) -> Result<(&'a mut [u8],usize),GenError> {
  let mut output = gen_slice!(x, &b"{"[..])?;
  let mut it = o.iter();

  if let Some((key, value)) = it.next() {
    output = gen_object_element(output, key, value, true)?;
  }

  for (key, value) in it {
    output = gen_object_element(output, key, value, false)?;
  }

  gen_slice!(output, &b"]"[..])
}

pub fn gen_object_element<'a>(x:(&'a mut [u8],usize), key: &String, value:&JsonValue, is_first: bool) -> Result<(&'a mut [u8],usize),GenError> {
  let mut output = x;
  if !is_first {
    output = gen_slice!(output, &b","[..])?;
  }

  do_gen!(output,
    gen_call!(gen_str, key) >>
    gen_slice!(&b":"[..]) >>
    gen_call!(gen_json_value, value)
  )
}

#[bench]
fn macros_json(b: &mut Bencher) {
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
    let (buf, index) = gen_json_value((&mut buffer, 0), &value).unwrap();

    println!("result:\n{}", str::from_utf8(buf).unwrap());
    //panic!();

    index as u64
  };

  println!("wrote {} bytes", index);
  b.bytes = index;
  b.iter(|| {
    let res = gen_json_value((&mut buffer, 0), &value).unwrap();
    res.1
  });
}

#[bench]
fn macros_gen_str(b: &mut Bencher) {

  let value = String::from("hello");
  let mut buffer = repeat(0).take(16384).collect::<Vec<u8>>();
  let index = {
    let (buf, index) = gen_str((&mut buffer, 0), &value).unwrap();

    println!("result:\n{}", str::from_utf8(buf).unwrap());
    //panic!();

    index as u64
  };

  println!("wrote {} bytes", index);
  b.bytes = index;
  b.iter(|| {
    let res = gen_str((&mut buffer, 0), &value).unwrap();
    res.1
  });
}

