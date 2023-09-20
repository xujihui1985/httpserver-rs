use std::{future::Future, net::{TcpStream, SocketAddr}, task::Poll, io};

use crate::async_io::reactor::REACTOR;

use super::client::TcpClient;

pub struct TcpListener {
    inner: std::net::TcpListener,
}

impl TcpListener {
    pub fn bind(addr: &str) -> std::io::Result<Self> {
        let listener = std::net::TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        Ok(Self {inner: listener})
    }

    pub fn accept(&self) -> Accept {
         
         REACTOR.with(|current| {
            let current = current.borrow();
            current.add(&self.inner);
         });

         Accept {
            listener: &self.inner
         }
    }
}

pub struct Accept<'a> {
    listener: &'a std::net::TcpListener
}

impl Future for Accept<'_> {
    type Output = std::io::Result<(TcpClient, SocketAddr)>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        match self.listener.accept() {
            Ok((stream, addr)) => Poll::Ready(Ok((TcpClient::new(stream), addr))),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                println!("would block");
                REACTOR.with(|current| {
                    let mut current = current.borrow_mut();
                    current.wake_on_readable(self.listener, cx);
                });
                Poll::Pending
            }
            Err(e) => Poll::Ready(Err(e))
        }
    }
}

impl Drop for Accept<'_> {
    fn drop(&mut self) {
        REACTOR.with(|current| {
            current.borrow_mut().remove(self.listener);
        });
    }
}
