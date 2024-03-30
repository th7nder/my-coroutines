use std::io::{ErrorKind, Read, Write};

use mio::{Interest, Token};

use crate::{future::{Future, PollState}, runtime::{self, reactor, Waker}};


fn get_req(path: &str) -> String {
    format!("GET {path} HTTP/1.1\r\n\
        Host: localhost\r\n\
        Connection: close\r\n\
        \r\n")
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
            id
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
    
    fn poll(&mut self, waker: &Waker) -> PollState<Self::Output> {
        if self.stream.is_none() {
            self.write_request();
            reactor().register(self.stream.as_mut().unwrap(), Interest::READABLE, self.id);
            reactor().set_waker(self.id, waker);
        }
        
        let mut buf = [0u8; 4096];
        
        loop {
            match self.stream.as_mut().unwrap().read(&mut buf) {
                Ok(0) => {
                    let str = String::from_utf8_lossy(&self.buffer);
                    // if we were to poll it one moar time, it'd probably fail?! 
                    reactor().deregister(self.stream.as_mut().unwrap(), self.id);
                    break PollState::Ready(str.to_string())
                }
                Ok(n) => {
                    self.buffer.extend(&buf[..n]);
                },
                Err(k) if k.kind() == ErrorKind::WouldBlock => {
                    reactor().set_waker(self.id, waker);
                    break PollState::NotReady
                },
                Err(k) if k.kind() == ErrorKind::Interrupted => {
                    continue;
                }
                Err(k) => panic!("{k:?}")
            }
        }
    }
}