extern crate cookie_factory;

#[path = "../tests/http/mod.rs"]
mod implementation;
use crate::implementation::*;
use async_std::net::TcpStream;

#[async_std::main]
async fn main() {
    use cookie_factory::async_bufwriter::{gen, AsyncBufWriter};

    let socket = TcpStream::connect("127.0.0.1:8080").await.unwrap();
    let stream = AsyncBufWriter::new(socket);

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
        .to_vec(),
        body: b"Hello, world!",
    };

    let sr = fn_request(&request);
    let (stream, size) = gen(sr, stream).await.unwrap();

    println!(
        "wrote:\n{} bytes (remaining in buffer: {})",
        size,
        stream.remaining()
    );
}
