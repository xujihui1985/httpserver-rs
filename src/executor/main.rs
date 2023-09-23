use async_io::executor::{self, EXECUTOR};
use async_net::listener::TcpListener;
use web::{router::Router, routes};

mod async_io;
mod async_net;
mod web;

fn main() {
    executor::block_on(async {
        let listen = TcpListener::bind("127.0.0.1:8088").unwrap();
        while let Ok((client, _addr)) = listen.accept().await {
            executor::spawn(async {
                let mut router = Router::new();
                routes::configure(&mut router);
                router.route_client(client).await.unwrap();
            });
        }
    })
    .unwrap();
}
