use gen::*;
use rewrite::*;

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
    .then(SliceSerializer::new(&r.body))
}

#[inline(always)]
pub fn rw_request_line<'a, S: AsRef<str>>(method: &'a S, uri: &'a S) -> impl Serializer + 'a {
  method.raw()
  .then(" ".raw())
  .then(uri.raw())
  .then(" HTTP/1.1\r\n".raw())
}

#[inline(always)]
pub fn rw_header<'a>(h: &'a Header) -> impl Serializer + 'a {
  h.name.raw()
    .then(": ".raw())
    .then(h.value.raw())
    .then("\r\n".raw())
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::str::from_utf8;

  #[test]
  fn cf() {
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

    assert_eq!(index, index2);
    assert_eq!(from_utf8(&s[..index]).unwrap(), from_utf8(&s2[..index2]).unwrap());
  }
}
