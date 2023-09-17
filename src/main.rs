use colored::*;
use std::net::TcpListener;
use std::sync::Arc;

use crate::thread_pool::ThreadPool;

mod response;
mod route_path;
mod router;
mod routes;
mod worker;
mod thread_pool;
mod nix;


fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8081")?;
    println!(
        "{} Server started",
        format!("[{}]", std::process::id()).green()
    );

    let mut router = router::Router::new();
    routes::configure(&mut router);
    let router = Arc::new(router);
    // let mut handlers = Vec::new();
    let pool = ThreadPool::new(4);
    for client in listener.incoming() {
        if let Ok(client) = client {
            let router = Arc::clone(&router);
            pool.execute(move || {
                router.route_client(client)
            });
            // let handle: JoinHandle<std::io::Result<()>>= std::thread::spawn(move || {
            //     println!("tid [{:?}] client connect", std::thread::current().id());
            //     router.route_client(client)?;
            //     Ok(())
            // });
            // handlers.push(handle);
        }
    }
    // while let Some(handle) = handlers.pop() {
    //     handle.join().unwrap()?;
    // }
    Ok(())
}
