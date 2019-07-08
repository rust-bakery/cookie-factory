extern crate cookie_factory;

#[path="../tests/http/mod.rs"] mod implementation;
use crate::implementation::*;

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


  let sr = fn_request(&request);
  let writer = cookie_factory::WriteCounter::new(vec![]);
  let writer = sr(writer).unwrap();
  let (buffer, size) = writer.into_inner();

  println!("result:\n{}", std::str::from_utf8(&buffer[..(size as usize)]).unwrap());
}

