extern crate cookie_factory;

use std::io::Write;
use cookie_factory::{Counter, SerializeFn, pair, string};

fn serializer<W: Write>() -> impl SerializeFn<W> {
  pair(string("Hello "), string("World!"))
}

fn main() {
  let s = {
    let mut c = Counter(0);
    let ser = serializer();
    match ser(&mut c) {
      Err(e) => {
        panic!("error calculating the length to serialize: {:?}", e);
      },
      Ok((_, s)) => {
        s
      }
    }
  };

  println!("length: {}", s);

  let mut v = Vec::with_capacity(s);

  let len = {
    let ser = serializer();
    match ser(&mut v) {
      Err(e) => {
        panic!("error serializing: {:?}", e);
      },
      Ok((_, len)) => {
        len
      }
    }
  };

  println!("wrote '{}'", std::str::from_utf8(&v[..s]).unwrap());

}
