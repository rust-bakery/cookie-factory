//#![feature(trace_macros)]
#![feature(test)]
extern crate test;
#[macro_use]
extern crate cookie_factory;

use std::str;
use std::iter::repeat;
use std::collections::HashMap;

use cookie_factory::*;
use cookie_factory::rewrite::*;
use cookie_factory::http::*;
use test::Bencher;

#[bench]
fn cf_http_bench(b: &mut Bencher) {
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
  let index = {
    let (buf, index) = cf_request((&mut buffer, 0), &request).unwrap();

    println!("result:\n{}", str::from_utf8(buf).unwrap());

    index as u64
  };

  println!("wrote {} bytes", index);
  b.bytes = index;
  b.iter(|| {
    let res = cf_request((&mut buffer, 0), &request).unwrap();
    assert_eq!(res.1 as u64, index);
  });
}

#[bench]
fn rw_http_bench(b: &mut Bencher) {
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
  let index = {
    let mut sr = rw_request(&request);
    let (index, res) = sr.serialize(&mut buffer).unwrap();

    println!("result:\n{}", str::from_utf8(&buffer[..index]).unwrap());
    assert_eq!(res, Serialized::Done);

    index as u64
  };

  println!("wrote {} bytes", index);
  b.bytes = index;
  b.iter(|| {
    let mut sr = rw_request(&request);
    let (i, _) = sr.serialize(&mut buffer).unwrap();
    assert_eq!(i as u64, index);
  });
}

#[bench]
fn rw_http_create_serializer_bench(b: &mut Bencher) {
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

  b.iter(|| {
    rw_request(&request)
  });
}

#[bench]
fn rw_http_partial_bench(b: &mut Bencher) {
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

  let mut buffer = repeat(0).take(100).collect::<Vec<u8>>();
  b.bytes = 149;

  b.iter(|| {
    let mut sr = rw_request(&request);
    let (i, res) = sr.serialize(&mut buffer).unwrap();
    assert_eq!(i, 100);
    assert_eq!(res, Serialized::Continue);
    let (i, res) = sr.serialize(&mut buffer).unwrap();
    assert_eq!(i, 49);
    assert_eq!(res, Serialized::Done);
  });
}

