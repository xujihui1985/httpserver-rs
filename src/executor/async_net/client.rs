use std::{
    future::Future,
    io::{ErrorKind, Read, Write},
    net,
    task::Poll,
};

use crate::async_io::reactor::REACTOR;

pub struct TcpClient {
    stream: net::TcpStream,
}

impl TcpClient {
    pub fn new(stream: net::TcpStream) -> Self {
        Self { stream: stream }
    }

    pub fn read<'a, T>(&'a mut self, buffer: &'a mut T) -> ReadFuture
    where
        T: AsMut<[u8]>,
    {
        ReadFuture {
            stream: &mut self.stream,
            buffer: buffer.as_mut(),
        }
    }

    pub fn write<'a, T>(&'a mut self, buffer: &'a T) -> WriteFuture
    where
        T: AsRef<[u8]>,
    {
        WriteFuture {
            stream: &mut self.stream,
            buffer: buffer.as_ref(),
        }
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        self.stream.flush()
    }
    
}

pub struct ReadFuture<'a> {
    stream: &'a mut net::TcpStream,
    buffer: &'a mut [u8],
}

impl Future for ReadFuture<'_> {
    type Output = std::io::Result<usize>;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let state = self.get_mut();
        match state.stream.read(state.buffer) {
            Ok(n) => Poll::Ready(Ok(n)),
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                REACTOR.with(|current| {
                    current.borrow_mut().wake_on_readable(&*state.stream, cx);
                });
                Poll::Pending
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

pub struct WriteFuture<'a> {
    stream: &'a mut net::TcpStream,
    buffer: &'a [u8],
}

impl Future for WriteFuture<'_> {
    type Output = std::io::Result<usize>;

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let state = self.get_mut();
        match state.stream.write(state.buffer) {
            Ok(n) => Poll::Ready(Ok(n)),
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                REACTOR.with(|current| {
                    current.borrow_mut().wake_on_writable(&*state.stream, cx);
                });
                Poll::Pending
            }
            Err(e) => Poll::Ready(Err(e)),
        }
    }
}

impl Drop for TcpClient {
    fn drop(&mut self) {
        REACTOR.with(|current| {
            current.borrow_mut().remove(&self.stream);
        })
    }
}
