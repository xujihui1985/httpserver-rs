use router::Router;
use std::sync::{Arc, RwLock};

use crate::async_io::event_loop::EventLoop;
use crate::async_io::listener::AsyncTcpListener;
use crate::async_io::reactor::Reactor;

mod nix;
mod response;
mod route_path;
mod router;
mod routes;
mod thread_pool;
mod worker;
mod async_io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reactor = Arc::new(RwLock::new(Reactor::new()?));
    let mut router = Router::new();
    routes::configure(&mut router);
    let router = Arc::new(router);
    let listener = AsyncTcpListener::bind("127.0.0.1:8081", reactor.clone(), router)?;

    reactor.write().unwrap().register(listener.fd, listener);

    let mut event_loop = EventLoop::new(reactor);
    event_loop.run()?;
    Ok(())
}
