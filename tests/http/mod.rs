use std::str;

use cookie_factory::*;

#[derive(Debug,Clone,PartialEq)]
pub struct Request<'a> {
  pub method: &'a str,
  pub uri: &'a str,
  pub headers: Vec<Header<'a>>,
  pub body: &'a [u8],
}

#[derive(Debug,Clone,PartialEq)]
pub struct Header<'a> {
  pub name: &'a str,
  pub value: &'a str,
}

pub fn cf_request<'a, 'b, 'c>(i:(&'a mut [u8],usize), r: &'c Request<'b>) -> Result<(&'a mut [u8],usize),GenError> {
  do_gen!((i.0, i.1),
    gen_call!(cf_request_line, r.method, r.uri) >>
    gen_many!(&r.headers, cf_header) >>
    gen_slice!(b"\r\n") >>
    gen_slice!(r.body)
  )
}

pub fn cf_request_line<'a, 'b>(i:(&'a mut [u8],usize), method: &'b str, uri: &'b str) -> Result<(&'a mut [u8],usize),GenError> {
  do_gen!((i.0, i.1),
    gen_slice!(method.as_bytes()) >>
    gen_slice!(b" ") >>
    gen_slice!(uri.as_bytes()) >>
    gen_slice!(b" HTTP/1.1\r\n")
  )
}

pub fn cf_header<'a, 'b, 'c>(i:(&'a mut [u8],usize), h: &'c Header<'b>) -> Result<(&'a mut [u8],usize),GenError> {
  do_gen!((i.0, i.1),
    gen_slice!(h.name.as_bytes()) >>
    gen_slice!(b": ") >>
    gen_slice!(h.value.as_bytes()) >>
    gen_slice!(b"\r\n")
  )
}

pub fn rw_request<'a, 'b: 'a>(r: &'b Request<'a>) -> impl Serializer + 'a {
  rw_request_line(&r.method, &r.uri)
    .then(all(r.headers.iter().map(rw_header)))
    .then("\r\n".raw())
    .then(Slice::new(&r.body))
}

//#[inline(always)]
pub fn rw_request_line<'a, S: AsRef<str>>(method: &'a S, uri: &'a S) -> impl Serializer + 'a {
  method.raw()
  .then(" ".raw())
  .then(uri.raw())
  .then(" HTTP/1.1\r\n".raw())
}

//#[inline(always)]
pub fn rw_header<'a>(h: &'a Header) -> impl Serializer + 'a {
  h.name.raw()
    .then(": ".raw())
    .then(h.value.raw()
    .then("\r\n".raw()))
}
