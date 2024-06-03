use std::fs::{self, File};
use std::io::Write;

use crate::http::{self, Request, Response};
use crate::server::Router;

pub fn add_handlers(http_router: &mut Router) {
    http_router.add_route(http::GET, "/", handle_root);
    http_router.add_route(http::GET, "/user-agent", handle_user_agent);
    http_router.add_route(http::GET, "/echo/:text", handle_echo);
    http_router.add_route(http::GET, "/files/:file_name", handle_file);
    http_router.add_route(http::POST, "/files/:file_name", handle_create_file);
}

fn handle_root(_request: Request, mut response: Response) {
    response.ok(None);
}

fn handle_echo(request: Request, mut response: Response) {
    let path_var = request.path_vars.get(":text").unwrap();

    response.set_header(String::from("Content-Type"), String::from("text/plain"));
    response.ok(Some(path_var.clone()));
}

fn handle_user_agent(request: Request, mut response: Response) {
    let user_agent = request.headers.get("User-Agent");
    response.set_header(String::from("Content-Type"), String::from("text/plain"));
    response.ok(Some(user_agent.unwrap_or(&String::from("")).to_string()));
}

fn handle_file(request: Request, mut response: Response) {
    let mut path = String::new();
    let file_name = request.path_vars.get(":file_name");

    if let Some(request_path) = &request.root_dir {
        path.push_str(&request_path);
        path.push_str(file_name.unwrap());

        match fs::read_to_string(path) {
            Ok(content) => {
                response.set_header(
                    "Content-Type".to_string(),
                    "application/octet-stream".to_string(),
                );
                response.ok(Some(content));
            }
            Err(err) => {
                eprintln!("Couldn't retrieve file content {}", err);
            }
        }
    }

    response.not_found();
}

fn handle_create_file(request: Request, mut response: Response) {
    let file_name = request.path_vars.get(":file_name");
    if let Some(mut full_path) = request.root_dir {
        full_path.push_str(file_name.unwrap());
        match File::create(full_path) {
            Ok(mut file) => {
                let _ = file.write_all(request.body.unwrap().as_bytes());
                response.no_content();
            }
            Err(err) => {
                eprintln!("error creating file {}", err);
                response.internal_server_error(None);
            }
        }
    }
}
