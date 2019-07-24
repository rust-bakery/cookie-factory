extern crate cookie_factory;

use std::io::{Write, sink};
use cookie_factory::{SerializeFn, pair, string, gen, gen_simple};

fn serializer<W: Write>() -> impl SerializeFn<W> {
  pair(string("Hello "), string("World!"))
}

fn main() {
  let s = {
    let c = sink();
    let ser = serializer();
    match gen(ser, c) {
      Err(e) => {
        panic!("error calculating the length to serialize: {:?}", e);
      },
      Ok((_, pos)) => {
        pos as usize
      }
    }
  };

  println!("length: {}", s);


  let v = {
    let v = Vec::with_capacity(s);
    let ser = serializer();
    match gen_simple(ser, v) {
      Err(e) => {
        panic!("error serializing: {:?}", e);
      },
      Ok(v) => {
        assert_eq!(v.len(), s);
        v
      }
    }
  };

  println!("wrote '{}'", std::str::from_utf8(&v[..s]).unwrap());

}
