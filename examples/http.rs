#![feature(test)]
extern crate test;
extern crate cookie_factory;

use std::iter::repeat;

#[path="../tests/http/mod.rs"] mod implementation;
use implementation::*;

use cookie_factory::Serializer;


//use cookie_factory::rewrite::*;
//use cookie_factory::http::*;

fn main() {
  let request = Request {
    method: "GET",
    uri: "/hello/test/a/b/c?name=value#hash",
    headers: [
      Header { name: "Host", value: "lolcatho.st" },
      Header { name: "User-agent", value: "cookie-factory" },
      Header { name: "Content-Length", value: "13" },
      Header { name: "Connection", value: "Close" },
    ].iter().cloned().collect(),
    body: b"Hello, world!",
  };

  let mut buffer = repeat(0).take(16384).collect::<Vec<u8>>();
  loop {
    let mut sr = rw_request(&request);
    let (index, res) = sr.serialize(&mut buffer).unwrap();
  }
}

