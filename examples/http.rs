extern crate cookie_factory;

use std::{str, iter::repeat};

#[path="../tests/http/mod.rs"] mod implementation;
use implementation::*;

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

  let size = {
    let sr = fn_request(&request);
    let (_buf, size) = sr(&mut buffer).unwrap();
    size
  };

  println!("result:\n{}", str::from_utf8(&buffer[..size]).unwrap());
}

