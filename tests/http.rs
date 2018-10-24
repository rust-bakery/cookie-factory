#![feature(test)]
extern crate test;
extern crate cookie_factory;

use cookie_factory::*;

#[path="../tests/http/mod.rs"] mod implementation;
use implementation::*;

#[cfg(test)]
mod tests {
  use super::*;
  use std::str::from_utf8;

  #[test]
  fn request() {
    let mut mem: [u8; 1024] = [0; 1024];
    let s = &mut mem[..];

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

    let (_, index) = cf_request((s, 0), &request).unwrap();
    println!("request written by cf:\n{}", from_utf8(&s[..index]).unwrap());

    let mut mem2: [u8; 1024] = [0; 1024];
    let s2 = &mut mem2[..];

    let mut sr = rw_request(&request);
    let (index2, res) = sr.serialize(s2).unwrap();
    assert_eq!(res, Serialized::Done);
    println!("request written by cf:\n{}", from_utf8(&s[..index]).unwrap());
    println!("wrote {} bytes", index2);

    assert_eq!(index, index2);
    assert_eq!(from_utf8(&s[..index]).unwrap(), from_utf8(&s2[..index2]).unwrap());
  }
}

