use crate::http::{self, Request, Response};
use crate::router::Router;

pub fn add_handlers(http_router: &mut Router) {
    http_router.add_route(http::GET, "/", handle_root);
    http_router.add_route(http::GET, "/echo/:text", handle_echo);
}

fn handle_root(_request: Request, mut response: Response) {
    response.ok(None);
}

fn handle_echo(request: Request, mut response: Response) {
    let path_var = request.path_vars.get(":text").unwrap();

    response.set_header(String::from("Content-Type"), String::from("text/plain"));
    response.ok(Some(path_var.clone()));
}

pub fn handle_not_found(mut response: Response) {
    response.not_found();
}
