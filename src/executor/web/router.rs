use std::{collections::HashMap, pin::Pin, future::Future};

use crate::{async_io::task_queue::LocalBoxedFuture, async_net::client::TcpClient};

use super::{node::Node, response::Response};

#[derive(Hash, PartialEq, Eq)]
pub enum Method {
    GET,
}

pub type HandlerFn = Pin<Box<dyn Fn(TcpClient) -> LocalBoxedFuture<'static, std::io::Result<()>>>>;

pub struct Router {
    routers: HashMap<Method, Node>,
}

impl Router {
    pub fn new() -> Self {
        Router {
            routers: HashMap::new(),
        }
    }

    pub fn insert<F, Fut>(&mut self, method: Method, path: &str, handler: F)
    where
        F: Fn(TcpClient) -> Fut + 'static,
        Fut: Future<Output = std::io::Result<()>> + 'static,
    {
        let node = self.routers.entry(method).or_insert(Node::new("/"));
        node.insert(path, Box::pin(move |client| Box::pin(handler(client))));
    }

    pub async fn route_client(&mut self, mut client: TcpClient) -> std::io::Result<()> {
        let mut buffer = [0; 1024];
        let n = client.read(&mut buffer).await?;

        let req = String::from_utf8_lossy(&buffer[..n]);
        let mut lines = req.split('\n');
        let line = lines.next().unwrap();
        let parts: Vec<&str> = line.split(" ").collect();
        if parts.len() < 2 {
            self.bad_request(client).await
        } else {
            match(parts[0], parts[1]) {
                ("GET", path) => self.handle(Method::GET, path, client).await,
                _ => self.not_found(client).await,
            }
        }
    }

    pub async fn handle(&mut self, method: Method, path: &str, client: TcpClient) -> std::io::Result<()> {
        let node = self.routers.get(&method);
        match node {
            Some(node) => {
                let handler = node.get(path);
                match handler {
                    Some(handler) => handler(client).await,
                    None => self.not_found(client).await,
                }
            }
            None => self.not_found(client).await,
        }
    }

    pub async fn bad_request(&self, client: TcpClient) -> std::io::Result<()> {
        let mut res = Response::new(client);
        res.send_file(400, "static/400.html").await?;
        Ok(())
    }

    pub async fn not_found(&self, client: TcpClient) -> std::io::Result<()> {
        let mut res = Response::new(client);
        res.send_file(400, "static/404.html").await?;
        Ok(())
    }
}
