use std::fs::{self, File};
use std::io::Write;
use std::sync::Arc;

use crate::http::{self, Request, Response};
use crate::server::Router;

pub type AsyncHandler = Arc<
    dyn Fn(Request, Response) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>
        + Send
        + Sync,
>;

pub fn add_handlers(http_router: &mut Router) {
    http_router.add_route(http::GET, "/", handle_root);
    http_router.add_route(http::GET, "/user-agent", handle_user_agent);
    http_router.add_route(http::GET, "/echo/:text", handle_echo);
    http_router.add_route(http::GET, "/files/:file_name", handle_file);
    http_router.add_route(http::POST, "/files/:file_name", handle_create_file);
}

async fn handle_root(request: Request, mut response: Response) {
    response.ok(request, None).await;
}

async fn handle_echo(request: Request, mut response: Response) {
    let path_vars = request.path_vars.clone();
    let path_var = path_vars.get(":text").unwrap();

    response.set_header(String::from("Content-Type"), String::from("text/plain"));
    response.ok(request, Some(path_var.clone())).await;
}

async fn handle_user_agent(request: Request, mut response: Response) {
    let path_vars = request.headers.clone();
    let user_agent = path_vars.get("User-Agent");
    response.set_header(String::from("Content-Type"), String::from("text/plain"));
    response
        .ok(
            request,
            Some(user_agent.unwrap_or(&String::from("")).to_string()),
        )
        .await;
}

async fn handle_file(request: Request, mut response: Response) {
    let mut path = String::new();
    let file_name = request.path_vars.get(":file_name");

    if let Some(request_path) = request.root_dir.clone() {
        path.push_str(&request_path);
        path.push_str(file_name.unwrap());

        match fs::read_to_string(path) {
            Ok(content) => {
                response.set_header(
                    "Content-Type".to_string(),
                    "application/octet-stream".to_string(),
                );
                response.ok(request, Some(content)).await;
                return;
            }
            Err(err) => {
                eprintln!("Couldn't retrieve file content {}", err);
            }
        }
    }

    response.not_found(request).await;
}

async fn handle_create_file(request: Request, mut response: Response) {
    let file_name = request.path_vars.get(":file_name");
    if let Some(mut full_path) = request.root_dir.clone() {
        full_path.push_str(file_name.unwrap());
        match File::create(full_path) {
            Ok(mut file) => {
                let _ = file.write_all(request.body.clone().unwrap().as_bytes());
                response.no_content(request).await;
            }
            Err(err) => {
                eprintln!("error creating file {}", err);
                response.internal_server_error(request, None).await;
            }
        }
    }
}
