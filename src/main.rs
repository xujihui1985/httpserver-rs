use colored::*;
use polling::{Event, Poller, Events};
use std::collections::HashMap;
use std::net::TcpListener;
use std::os::fd::AsRawFd;
use std::sync::Arc;

use crate::thread_pool::ThreadPool;

mod nix;
mod response;
mod route_path;
mod router;
mod routes;
mod thread_pool;
mod worker;
mod async_io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8081")?;
    // set non-blocking
    listener.set_nonblocking(true)?;

    let listener_fd = listener.as_raw_fd() as usize;
    // create a poller
    let poller = Poller::new()?;
    // register the listener
    unsafe {
        poller.add(
            &listener, 
            Event::readable(listener_fd)
        )?;
    }
 
    println!(
        "{} Server started",
        format!("[{}]", std::process::id()).green()
    );

    let mut router = router::Router::new();
    let pool = ThreadPool::new(4);
    routes::configure(&mut router);
    let router = Arc::new(router);
    let mut clients = HashMap::new();
    let mut events = Events::new();
    loop {
        events.clear();
        poller.wait(&mut events, None)?;

        for ev in events.iter() {
            if ev.key == listener_fd && ev.readable {
                let (client, _) = listener.accept()?;
                let client_fd = client.as_raw_fd() as usize;
                unsafe {poller.add(&client, Event::readable(client_fd))?;}
                clients.insert(client_fd, client);
                poller.modify(&listener, Event::readable(listener_fd))?;
            } else if ev.readable && clients.contains_key(&ev.key) {
                if let Some(client) = clients.remove(&ev.key) {
                    let router = Arc::clone(&router);
                    pool.execute(move || {
                        router.route_client(client)?;
                        Ok(())
                    });
                }
            }
        }
    }
}
