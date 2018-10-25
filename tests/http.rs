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

  #[test]
  fn chunked_http() {
    let mut mem: [u8; 1024] = [0; 1024];
    let s = &mut mem[..];

    let request = RequestHeaders {
      method: "GET",
      uri: "/hello/test/a/b/c?name=value#hash",
      headers: [
        Header { name: "Host", value: "lolcatho.st" },
        Header { name: "User-agent", value: "cookie-factory" },
        Header { name: "Content-Length", value: "13" },
        Header { name: "Connection", value: "Close" },
      ].iter().cloned().collect(),
    };

    let mut sr = chunked_request(&request);
    assert_eq!(sr.serialize(&mut s[..132]), Ok((132, Serialized::Continue)));

    // add chunk
    sr.second.push(chunk(&b"Hello "[..]));
    assert_eq!(sr.serialize(&mut s[132..145]), Ok((13, Serialized::Continue)));
    assert_eq!(from_utf8(&s[132..145]).unwrap(), "\r\n\r\n6\r\nHello ");

    // add chunk
    sr.second.push(chunk(&b"world !"[..]));
    // add last chunk
    sr.second.push(chunk(&[]));

    assert_eq!(sr.serialize(&mut s[145..]), Ok((19, Serialized::Done)));
    assert_eq!(from_utf8(&s[136..164]).unwrap(), "6\r\nHello \r\n7\r\nworld !\r\n0\r\n\r\n");

  }
}

