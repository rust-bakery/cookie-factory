use std::io::Write;
use std::str;

use cookie_factory::combinator::{hex, slice, string};
use cookie_factory::multi::all;
use cookie_factory::sequence::tuple;
use cookie_factory::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Request<'a> {
    pub method: &'a str,
    pub uri: &'a str,
    pub headers: Vec<Header<'a>>,
    pub body: &'a [u8],
}

#[derive(Debug, Clone, PartialEq)]
pub struct Header<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

pub fn cf_request<'a, 'b, 'c>(
    i: (&'a mut [u8], usize),
    r: &'c Request<'b>,
) -> Result<(&'a mut [u8], usize), GenError> {
    do_gen!(
        (i.0, i.1),
        gen_call!(cf_request_line, r.method, r.uri)
            >> gen_many!(&r.headers, cf_header)
            >> gen_slice!(b"\r\n")
            >> gen_slice!(r.body)
    )
}

pub fn cf_request_line<'a, 'b>(
    i: (&'a mut [u8], usize),
    method: &'b str,
    uri: &'b str,
) -> Result<(&'a mut [u8], usize), GenError> {
    do_gen!(
        (i.0, i.1),
        gen_slice!(method.as_bytes())
            >> gen_slice!(b" ")
            >> gen_slice!(uri.as_bytes())
            >> gen_slice!(b" HTTP/1.1\r\n")
    )
}

pub fn cf_header<'a, 'b, 'c>(
    i: (&'a mut [u8], usize),
    h: &'c Header<'b>,
) -> Result<(&'a mut [u8], usize), GenError> {
    do_gen!(
        (i.0, i.1),
        gen_slice!(h.name.as_bytes())
            >> gen_slice!(b": ")
            >> gen_slice!(h.value.as_bytes())
            >> gen_slice!(b"\r\n")
    )
}

#[derive(Debug, Clone, PartialEq)]
pub struct RequestHeaders<'a> {
    pub method: &'a str,
    pub uri: &'a str,
    pub headers: Vec<Header<'a>>,
}

pub fn fn_request<'a: 'c, 'b: 'a, 'c, W: Write + 'c>(
    r: &'b Request<'a>,
) -> impl SerializeFn<W> + 'c {
    tuple((
        fn_request_line(&r.method, &r.uri),
        all(r.headers.iter().map(fn_header)),
        string("\r\n"),
        slice(r.body),
    ))
}

pub fn fn_request_line<'a: 'c, 'c, S: AsRef<str>, W: Write + 'c>(
    method: &'a S,
    uri: &'a S,
) -> impl SerializeFn<W> + 'c {
    tuple((
        string(method),
        string(" "),
        string(uri),
        string(" HTTP/1.1\r\n"),
    ))
}

pub fn fn_header<'a: 'c, 'c, W: Write + 'c>(h: &'a Header) -> impl SerializeFn<W> + 'c {
    tuple((
        string(h.name),
        string(": "),
        string(h.value),
        string("\r\n"),
    ))
}

pub fn fn_request_headers<'a: 'c, 'c, 'b: 'a, W: Write + 'c>(
    r: &'b RequestHeaders<'a>,
) -> impl SerializeFn<W> + 'c {
    tuple((
        fn_request_line(&r.method, &r.uri),
        all(r.headers.iter().map(fn_header)),
        string("\r\n"),
    ))
}

pub fn fn_chunk<'a: 'c, 'c, W: Write + 'c>(sl: &'a [u8]) -> impl SerializeFn<W> + 'c {
    tuple((hex(sl.len()), string("\r\n"), slice(sl), string("\r\n")))
}

/*
pub fn chunked_request<'a, 'b: 'a, 'c, T: Serializer + 'a>(r: &'b RequestHeaders<'a>) -> Then<impl Serializer + 'a, Stream<T>> {
  let s: Stream<T> = Stream::new();
  rw_request_headers(r).then(s)
}*/
