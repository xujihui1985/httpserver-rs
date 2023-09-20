use async_io::executor::{self, EXECUTOR};
use async_net::listener::TcpListener;
use web::{router::Router, routes};

mod async_io;
mod async_net;
mod web;

fn main() {
    let mut execu = executor::Executor::new();
    // execu.spawn(async {
    //     let x = test().await;
    //     println!("{}", x);
    // });

    execu.spawn(async {
        let listen = TcpListener::bind("127.0.0.1:8088").unwrap();
        let mut router = Router::new();
        routes::configure(&mut router);
        while let Ok((client, addr)) = listen.accept().await {
            executor::spawn(async {
                router.route_client(client).await.unwrap();
            })
        }
    });

    execu.run();
}

async fn test() -> usize {
    5
}
