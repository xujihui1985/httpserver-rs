use std::{
    collections::HashMap,
    hash::Hash,
    io::{BufRead, BufReader},
    net::TcpStream,
};

use crate::{response::Response, route_path::Node};

#[derive(Hash, PartialEq, Eq)]
pub enum Method {
    GET,
}

pub struct Router {
    routers: HashMap<Method, Node>,
}

impl Router {
    pub fn new() -> Self {
        Router {
            routers: HashMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        method: Method,
        path: &str,
        handler: fn(TcpStream) -> std::io::Result<()>,
    ) {
        let node = self.routers.entry(method).or_insert(Node::new("/"));
        node.insert(path, handler);
    }

    pub fn route_client(&self, client: TcpStream) -> std::io::Result<()> {
        let mut reader = BufReader::new(&client);
        let buf = reader.fill_buf()?;

        // read a single line
        let mut line = String::new();
        let mut line_reader = BufReader::new(buf);
        let len = line_reader.read_line(&mut line)?;

        // consume bytes
        reader.consume(len);
        if len == 0 {
            return Ok(());
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            let mut res = Response::new(client);
            return res.sendfile(400, "static/400.html");
        }
        match (parts[0], parts[1]) {
            ("GET", path) => self.handle(Method::GET, path, client),
            _ => self.not_found(client),
        }
    }

    pub fn handle(&self, method: Method, resource: &str, client: TcpStream) -> std::io::Result<()> {
        if let Some(node) = self.routers.get(&method) {
            if let Some(handler) = node.get(resource) {
                return handler(client);
            }
        }
        self.not_found(client)
    }

    pub fn not_found(&self, client: TcpStream) -> std::io::Result<()> {
        let mut res = Response::new(client);
        res.sendfile(404, "static/404.html")
    }
}
