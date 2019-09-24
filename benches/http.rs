#![feature(test)]
extern crate cookie_factory;
extern crate test;

use std::iter::repeat;

use test::Bencher;

#[path = "../tests/http/mod.rs"]
mod implementation;
use crate::implementation::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::from_utf8;

    #[test]
    fn macros() {
        let mut mem: [u8; 1024] = [0; 1024];
        let s = &mut mem[..];

        let request = Request {
            method: "GET",
            uri: "/hello/test/a/b/c?name=value#hash",
            headers: [
                Header {
                    name: "Host",
                    value: "lolcatho.st",
                },
                Header {
                    name: "User-agent",
                    value: "cookie-factory",
                },
                Header {
                    name: "Content-Length",
                    value: "13",
                },
                Header {
                    name: "Connection",
                    value: "Close",
                },
            ]
            .iter()
            .cloned()
            .collect(),
            body: b"Hello, world!",
        };

        let (_, index) = cf_request((s, 0), &request).unwrap();
        println!(
            "request written by cf:\n{}",
            from_utf8(&s[..index]).unwrap()
        );
    }
}

mod macros {
    use super::*;
    use std::str;

    #[bench]
    fn http(b: &mut Bencher) {
        let request = Request {
            method: "GET",
            uri: "/hello/test/a/b/c?name=value#hash",
            headers: [
                Header {
                    name: "Host",
                    value: "lolcatho.st",
                },
                Header {
                    name: "User-agent",
                    value: "cookie-factory",
                },
                Header {
                    name: "Content-Length",
                    value: "13",
                },
                Header {
                    name: "Connection",
                    value: "Close",
                },
            ]
            .iter()
            .cloned()
            .collect(),
            body: b"Hello, world!",
        };

        let mut buffer = repeat(0).take(16384).collect::<Vec<u8>>();
        let index = {
            let (buf, index) = cf_request((&mut buffer[..], 0), &request).unwrap();

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
}

mod functions {
    use super::*;
    use cookie_factory::{gen, gen_simple};

    #[bench]
    fn http(b: &mut Bencher) {
        let request = Request {
            method: "GET",
            uri: "/hello/test/a/b/c?name=value#hash",
            headers: [
                Header {
                    name: "Host",
                    value: "lolcatho.st",
                },
                Header {
                    name: "User-agent",
                    value: "cookie-factory",
                },
                Header {
                    name: "Content-Length",
                    value: "13",
                },
                Header {
                    name: "Connection",
                    value: "Close",
                },
            ]
            .iter()
            .cloned()
            .collect(),
            body: b"Hello, world!",
        };

        let mut buffer = repeat(0).take(16384).collect::<Vec<u8>>();
        let index = {
            let sr = fn_request(&request);
            let (_, pos) = gen(sr, &mut buffer[..]).unwrap();

            //println!("result:\n{}", str::from_utf8(buf).unwrap());

            pos as usize
        };

        println!("wrote {} bytes", index);
        b.bytes = index as u64;
        b.iter(|| {
            let sr = fn_request(&request);
            let _ = gen_simple(sr, &mut buffer[..]).unwrap();
        });
    }
}
