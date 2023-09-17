use std::{net::{TcpListener, TcpStream}, os::fd::{AsFd, AsRawFd}};

pub struct AsyncTcpListener {
    pub listener: TcpListener,
    fd: usize,
    pub state: Option<ListenerState>
}

pub enum ListenerState {
   Waiting, 
   Accepting(TcpStream),
   Closed
}

impl AsyncTcpListener {
    pub fn bind(addr: &str) -> std::io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        let fd = listener.as_raw_fd() as usize;
        Ok(AsyncTcpListener { 
            listener ,
            fd,
            state: Some(ListenerState::Waiting)
        })
    }
}