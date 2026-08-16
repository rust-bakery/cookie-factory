use std::collections::BTreeMap;
use std::str;

use cookie_factory::{
    combinator::string, lib::std::io::Write, multi::separated_list, sequence::tuple, *,
};

#[derive(Clone, Debug, PartialEq)]
pub enum JsonValue {
    Str(String),
    Boolean(bool),
    Num(f64),
    Array(Vec<JsonValue>),
    Object(BTreeMap<String, JsonValue>),
}

#[inline(always)]
pub fn jen_str<'a, 'b: 'a, W: Write + 'a>(s: &'b str) -> impl SerializeFn<W> + 'a {
    tuple((string("\""), string(s), string("\"")))
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
    string("1234.56")
}

pub fn jen_array<'a, 'b: 'a, W: Write + 'a>(arr: &'b [JsonValue]) -> impl SerializeFn<W> + 'a {
    tuple((
        string("["),
        separated_list(string(","), arr.iter().map(jen_json_value)),
        string("]"),
    ))
}

pub fn jen_key_value<'a, 'b: 'a, W: Write + 'a>(
    kv: (&'b String, &'b JsonValue),
) -> impl SerializeFn<W> + 'a {
    tuple((jen_str(kv.0), string(":"), jen_json_value(kv.1)))
}

pub fn jen_object<'a, 'b: 'a, W: Write + 'a>(
    o: &'b BTreeMap<String, JsonValue>,
) -> impl SerializeFn<W> + 'a {
    tuple((
        string("{"),
        separated_list(string(","), o.iter().map(jen_key_value)),
        string("}"),
    ))
}

pub fn jen_json_value<W: Write>(g: &JsonValue) -> impl SerializeFn<W> + '_ {
    move |out: WriteContext<W>| match g {
        JsonValue::Str(ref s) => jen_str(s)(out),
        JsonValue::Boolean(ref b) => jen_bool(*b)(out),
        JsonValue::Num(ref n) => jen_num(*n)(out),
        JsonValue::Array(ref v) => jen_array(v)(out),
        JsonValue::Object(ref o) => jen_object(o)(out),
    }
}

#[test]
fn json_test() {
    use cookie_factory::lib::std::io::Cursor;
    use std::str;

    let value = JsonValue::Object(btreemap! {
      String::from("arr") => JsonValue::Array(vec![JsonValue::Num(1.0), JsonValue::Num(12.3), JsonValue::Num(42.0)]),
      String::from("b") => JsonValue::Boolean(true),
      String::from("o") => JsonValue::Object(btreemap!{
        String::from("x") => JsonValue::Str(String::from("abcd")),
        String::from("y") => JsonValue::Str(String::from("efgh")),
        String::from("empty") => JsonValue::Array(vec![]),
      }),
    });

    let mut buffer = [0u8; 8192];
    let sr = jen_json_value(&value);
    let cursor = Cursor::new(&mut buffer[..]);
    let cursor = jen_simple(sr, cursor).unwrap();
    let size = cursor.position() as usize;
    let buffer = cursor.into_inner();

    println!("result:\n{}", str::from_utf8(&buffer[..size]).unwrap());
    assert_eq!(str::from_utf8(&buffer[..size]).unwrap(),
    "{\"arr\":[1234.56,1234.56,1234.56],\"b\":true,\"o\":{\"empty\":[],\"x\":\"abcd\",\"y\":\"efgh\"}}");
}
