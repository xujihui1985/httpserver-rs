use super::{router::Router, response::Response};

pub fn configure(router: &mut Router) {
    router.insert(super::router::Method::GET, "/", |client| async {
        let mut res = Response::new(client);
        res.send_file(200, "static/index.html").await
    })
}