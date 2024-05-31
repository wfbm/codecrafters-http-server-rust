use crate::http::{self, Request, Response};
use crate::router::Router;
use std::io::Write;

pub fn add_handlers(http_router: &mut Router) {
    http_router.add_route(http::GET, "/", handle_root);
    http_router.add_route(http::GET, "/echo/:text", handle_echo);
}

fn handle_root(_request: Request, mut response: Response) {
    let _ = response.conn.write_all(http::OK_RESPONSE.as_bytes());
}

fn handle_echo(request: Request, mut response: Response) {
    let mut response_str = String::new();

    let path_var = request.path_vars.get(":text").unwrap();
    response_str.push_str(http::OK_RESPONSE);
    response_str.push_str(&path_var);
    response_str.push_str("\r\n");

    let _ = response.conn.write_all(response_str.as_bytes());
}
