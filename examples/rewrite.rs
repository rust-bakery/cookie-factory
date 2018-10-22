#![recursion_limit="128"]
//#![feature(trace_macros)]
#![feature(test)]
extern crate test;
//#[macro_use]
extern crate cookie_factory;

use std::str;
use std::iter::repeat;
use std::collections::HashMap;

use cookie_factory::*;
use cookie_factory::rewrite::*;

// from https://github.com/bluss/maplit
macro_rules! hashmap {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(hashmap!(@single $rest)),*]));

    ($($key:expr => $value:expr,)+) => { hashmap!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let _cap = hashmap!(@count $($key),*);
            let mut _map = ::std::collections::HashMap::with_capacity(_cap);
            $(
                let _ = _map.insert($key, $value);
            )*
            _map
        }
    };
}


fn main() {
  let element = JsonValue::Object(hashmap!{
    String::from("arr") => JsonValue::Array(vec![JsonValue::Num(1.0), JsonValue::Num(12.3), JsonValue::Num(42.0)]),
    String::from("b") => JsonValue::Boolean(true),
    String::from("o") => JsonValue::Object(hashmap!{
      String::from("x") => JsonValue::Str(String::from("abcd")),
      String::from("y") => JsonValue::Str(String::from("efgh")),
      String::from("empty") => JsonValue::Array(vec![]),
    }),
  });

  let value = JsonValue::Array(repeat(element).take(10).collect::<Vec<JsonValue>>());
  let mut buffer = repeat(0).take(16384).collect::<Vec<u8>>();
  loop {
    let mut sr = gen_json_value(&value);

    let (index, result) = sr.serialize(&mut buffer).unwrap();

    //println!("result:\n{}", str::from_utf8(&buffer[..index]).unwrap());
  }
}

