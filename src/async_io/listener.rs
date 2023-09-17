use std::{net::{TcpListener, TcpStream}, os::fd::{AsFd, AsRawFd}, sync::{Arc, RwLock}};

use polling::Event;

use crate::router::Router;

use super::{reactor::Reactor, event_handler::EventHandler, client::ClientRequest};

pub struct AsyncTcpListener {
    pub listener: TcpListener,
    pub fd: usize,
    pub state: Option<ListenerState>,
    reactor: Arc<RwLock<Reactor>>,
    router: Arc<Router>,
}

pub enum ListenerState {
   Waiting, 
   Accepting(TcpStream),
   Closed
}

impl AsyncTcpListener {
    pub fn bind(addr: &str, reactor: Arc<RwLock<Reactor>>, router: Arc<Router>) -> std::io::Result<Self> {
        let listener = TcpListener::bind(addr)?;
        listener.set_nonblocking(true)?;
        let fd = listener.as_raw_fd() as usize;
        {
            let reactor_guard = reactor.write().unwrap();
            reactor_guard.add(&listener, Event::readable(fd))?;
        }
        Ok(AsyncTcpListener { 
            listener ,
            fd,
            reactor,
            state: Some(ListenerState::Waiting),
            router,
        })
    }
}

impl EventHandler for AsyncTcpListener {
    fn id(&self) -> usize {
        self.fd
    }

    fn name(&self) -> String {
        format!("TcpListener {}", self.listener.local_addr().unwrap())
    }

    fn event(&mut self, event: Event) {
        if event.readable {
            let (client, addr) = self.listener.accept().unwrap();
            // at this point the listener state is becoming Accepting
            self.state.replace(ListenerState::Accepting(client));
            let mut reactor = self.reactor.write().unwrap();
            reactor.schedule(self.fd);
            
        }
    }

    fn poll(&mut self) {
        match self.state.take() {
            Some(ListenerState::Accepting(client)) => {
                let mut reactor = self.reactor.write().unwrap();
                reactor.modify(&self.listener, Event::readable(self.id())).unwrap();
                self.state.replace(ListenerState::Waiting);
                let router = self.router.clone();
                let client = ClientRequest::new(client, router, self.reactor.clone());
                reactor.register(client.fd, client);
            }
            _ => {}
        }
    }
}