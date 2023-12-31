use std::net::TcpStream;

use crate::{
    response::Response,
    router::{Method, Router},
};

pub fn configure(router: &mut Router) {
    router.insert(Method::GET, "/", index);
}

fn index(client: TcpStream) -> std::io::Result<()> {
    let mut res = Response::new(client);
    res.sendfile(200, "static/index.html")
}
