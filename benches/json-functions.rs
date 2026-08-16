#![feature(test)]
extern crate cookie_factory;
extern crate test;
#[macro_use]
extern crate maplit;

use std::collections::BTreeMap;
use std::io::Write;
use std::str;
use test::Bencher;

use cookie_factory::*;
use cookie_factory::{combinator::string, multi::separated_list};

#[derive(Clone, Debug, PartialEq)]
pub enum JsonValue {
    Str(String),
    Boolean(bool),
    Num(f64),
    Array(Vec<JsonValue>),
    Object(BTreeMap<String, JsonValue>),
}

#[inline(always)]
pub fn jen_str<'a, 'b: 'a, W: Write>(s: &'b str) -> impl SerializeFn<W> + 'a {
    move |out: WriteContext<W>| {
        let out = string("\"")(out)?;
        let out = string(s)(out)?;
        string("\"")(out)
    }
}

#[inline(always)]
pub fn jen_bool<W: Write>(b: bool) -> impl SerializeFn<W> {
    if b {
        string("true")
    } else {
        string("false")
    }
}

#[inline(always)]
pub fn jen_num<W: Write>(_b: f64) -> impl SerializeFn<W> {
    /*move |out: &'a mut [u8]| {
      let s = format!("{}", b);
      string(s)(out)
    }*/
    string("1234.56")
}

pub fn jen_array<'a, 'b: 'a, W: Write>(arr: &'b [JsonValue]) -> impl SerializeFn<W> + 'a {
    move |out: WriteContext<W>| {
        let out = string("[")(out)?;
        let out = separated_list(string(","), arr.iter().map(jen_json_value))(out)?;
        string("]")(out)
    }
}

pub fn jen_key_value<'a, 'b: 'a, W: Write>(
    kv: (&'b String, &'b JsonValue),
) -> impl SerializeFn<W> + 'a {
    move |out: WriteContext<W>| {
        let out = jen_str(kv.0)(out)?;
        let out = string(":")(out)?;
        jen_json_value(kv.1)(out)
    }
}

pub fn jen_object<'a, 'b: 'a, W: Write>(
    o: &'b BTreeMap<String, JsonValue>,
) -> impl SerializeFn<W> + 'a {
    move |out: WriteContext<W>| {
        let out = string("{")(out)?;

        let out = separated_list(string(","), o.iter().map(jen_key_value))(out)?;
        string("}")(out)
    }
}

pub fn jen_json_value<'a, W: Write>(g: &'a JsonValue) -> impl SerializeFn<W> + 'a {
    move |out: WriteContext<W>| match g {
        JsonValue::Str(ref s) => jen_str(s)(out),
        JsonValue::Boolean(ref b) => jen_bool(*b)(out),
        JsonValue::Num(ref n) => jen_num(*n)(out),
        JsonValue::Array(ref v) => jen_array(v)(out),
        JsonValue::Object(ref o) => jen_object(o)(out),
    }
}

use std::iter::repeat_n;

#[test]
fn json_test() {
    let value = JsonValue::Object(btreemap! {
      String::from("arr") => JsonValue::Array(vec![JsonValue::Num(1.0), JsonValue::Num(12.3), JsonValue::Num(42.0)]),
      String::from("b") => JsonValue::Boolean(true),
      String::from("o") => JsonValue::Object(btreemap!{
        String::from("x") => JsonValue::Str(String::from("abcd")),
        String::from("y") => JsonValue::Str(String::from("efgh")),
        String::from("empty") => JsonValue::Array(vec![]),
      }),
    });

    let mut buffer = repeat_n(0, 16384).collect::<Vec<u8>>();
    let index = {
        let sr = jen_json_value(&value);

        let (_, res) = jen(sr, &mut buffer[..]).unwrap();
        res as usize
    };

    println!("result:\n{}", str::from_utf8(&buffer[..index]).unwrap());
    assert_eq!(str::from_utf8(&buffer[..index]).unwrap(),
    "{\"arr\":[1234.56,1234.56,1234.56],\"b\":true,\"o\":{\"empty\":[],\"x\":\"abcd\",\"y\":\"efgh\"}}");
    panic!();
}

#[bench]
fn functions_json(b: &mut Bencher) {
    let element = JsonValue::Object(btreemap! {
      String::from("arr") => JsonValue::Array(vec![JsonValue::Num(1.0), JsonValue::Num(12.3), JsonValue::Num(42.0)]),
      String::from("b") => JsonValue::Boolean(true),
      String::from("o") => JsonValue::Object(btreemap!{
        String::from("x") => JsonValue::Str(String::from("abcd")),
        String::from("y") => JsonValue::Str(String::from("efgh")),
        String::from("empty") => JsonValue::Array(vec![]),
      }),
    });

    let value = JsonValue::Array(repeat_n(element, 10).collect::<Vec<JsonValue>>());

    let mut buffer = repeat_n(0u8, 16384).collect::<Vec<_>>();
    let index = {
        let sr = jen_json_value(&value);

        let (_, res) = jen(sr, &mut buffer[..]).unwrap();
        res as usize
    };

    b.bytes = index as u64;
    b.iter(|| {
        let sr = jen_json_value(&value);
        let _ = jen_simple(sr, &mut buffer[..]).unwrap();
    });
}

#[bench]
fn functions_jen_str(b: &mut Bencher) {
    let mut buffer = repeat_n(0, 16384).collect::<Vec<u8>>();

    let index = {
        let sr = jen_str("hello");

        let (_, res) = jen(sr, &mut buffer[..]).unwrap();
        res as usize
    };

    b.bytes = index as u64;
    b.iter(|| {
        let sr = jen_str("hello");
        let _ = jen_simple(sr, &mut buffer[..]).unwrap();
    });
}
