use std::net::TcpStream;

pub type HandlerFn = fn(client: TcpStream) -> std::io::Result<(i32, String)>;

pub struct Node {
    pub nodes: Vec<Node>,
    pub key: String,
    pub handler: Option<HandlerFn>,
}

impl Node {
    pub fn new(key: &str) -> Self {
        Self {
            nodes: Vec::new(),
            key: key.to_string(),
            handler: None,
        }
    }

    pub fn insert(&mut self, path: &str, handler: HandlerFn) {
        // "/foo/bar"
        match path.split_once('/') {
            // "foo/"
            Some((root, "")) => {
                self.key = root.to_string();
                self.handler = Some(handler);
            }
            // "/foo"
            Some(("", path)) => self.insert(path, handler),
            // "foo/bar"
            Some((root, path)) => {
                let node = self.nodes.iter_mut().find(|m| root == &m.key);
                match node {
                    Some(node) => node.insert(path, handler),
                    None => {
                        let mut node = Node::new(root);
                        node.insert(path, handler);
                        self.nodes.push(node);
                    }
                }
            }
            None => {
                let mut node = Node::new(path);
                node.handler = Some(handler);
                self.nodes.push(node);
            }
        }
    }

    // get /root/bar
    pub fn get(&self, path: &str) -> Option<HandlerFn> {
        match path.split_once('/') {
            Some((root, "")) => {
                if root == &self.key {
                    self.handler
                } else {
                    None
                }
            }
            // "/foo"
            Some(("", path)) => self.get(path),
            // "foo/bar"
            Some((root, path)) => {
                let node = self.nodes.iter().find(|m| root == &m.key);
                if let Some(node) = node {
                    node.get(path)
                } else {
                    None
                }
            }
            None => {
                let node = self.nodes.iter().find(|m| path == &m.key);
                if let Some(node) = node {
                    node.handler
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_routers() {
        let mut root = Node::new("/");
        root.insert("/", |_| Ok((200, "static/index.html".to_string())));
        root.insert("/foo", |_| Ok((200, "static/index.html".to_string())));
        root.insert("/foo/bar", |_| Ok((200, "static/index.html".to_string())));

        assert!(root.get("/").is_some());
        assert!(root.get("/foo").is_some());
        assert!(root.get("/foo/bar").is_some());
        assert!(root.get("/foobar").is_none());
    }
}
