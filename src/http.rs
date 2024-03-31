use std::{
    future::Future, io::{ErrorKind, Read, Write}, pin::Pin, task::{Context, Poll}
};

use mio::{Interest, Token};

use crate::{
    runtime::{self, reactor, MyWaker},
};

fn get_req(path: &str) -> String {
    format!(
        "GET {path} HTTP/1.1\r\n\
        Host: localhost\r\n\
        Connection: close\r\n\
        \r\n"
    )
}

pub struct Http;

impl Http {
    pub fn get(path: &str) -> impl Future<Output = String> {
        HttpGetFuture::new(path)
    }
}

pub struct HttpGetFuture {
    stream: Option<mio::net::TcpStream>,
    buffer: Vec<u8>,
    path: String,
    id: usize,
}

impl HttpGetFuture {
    fn new(path: &str) -> HttpGetFuture {
        let id = reactor().next_id();
        HttpGetFuture {
            stream: None,
            buffer: Vec::new(),
            path: path.into(),
            id,
        }
    }

    fn write_request(&mut self) {
        let stream = std::net::TcpStream::connect("127.0.0.1:8080").unwrap();
        stream.set_nonblocking(true);
        let mut stream = mio::net::TcpStream::from_std(stream);
        stream.write_all(get_req(&self.path).as_bytes()).unwrap();
        self.stream = Some(stream);
    }
}

impl Future for HttpGetFuture {
    type Output = String;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let id = self.id;
        let this = self.get_mut();
        if this.stream.is_none() {
            this.write_request();
            let stream = this.stream.as_mut().unwrap();
            reactor().register(stream, Interest::READABLE, id);
            reactor().set_waker(id, cx);
        }

        let mut buf = [0u8; 4096];

        loop {
            match this.stream.as_mut().unwrap().read(&mut buf) {
                Ok(0) => {
                    let str = String::from_utf8_lossy(&this.buffer);
                    // if we were to poll it one moar time, it'd probably fail?!
                    reactor().deregister(this.stream.as_mut().unwrap(), id);
                    break Poll::Ready(str.to_string());
                }
                Ok(n) => {
                    this.buffer.extend(&buf[..n]);
                }
                Err(k) if k.kind() == ErrorKind::WouldBlock => {
                    reactor().set_waker(id, cx);
                    break Poll::Pending;
                }
                Err(k) if k.kind() == ErrorKind::Interrupted => {
                    continue;
                }
                Err(k) => panic!("{k:?}"),
            }
        }
    }
}
