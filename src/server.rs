use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};

use crate::handler::{self, AsyncHandler};
use crate::http;
use crate::http::{Request, Response};

pub struct Server {
    router: Router,
}

impl Server {
    pub async fn start(&self) {
        let listener = TcpListener::bind("127.0.0.1:4221")
            .await
            .expect("unable to bind to address");

        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let router = self.router.clone();
                    tokio::spawn(async move { handle_conn(stream, router).await });
                }
                Err(e) => {
                    println!("error: {}", e);
                }
            }
        }
    }
}

pub fn new_server(mut directory: String) -> Server {
    let dir = if !directory.is_empty() {
        if !directory.ends_with("/") {
            directory.push_str("/");
        }

        Some(directory)
    } else {
        None
    };

    let http_router = new_router(dir.clone());

    Server {
        router: http_router,
    }
}

async fn read(stream: &mut TcpStream) -> Result<String, std::io::Error> {
    let mut buffer = [0; 512];
    let mut request = String::new();

    loop {
        let bytes_read = stream.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }

        request.push_str(&String::from_utf8_lossy(&buffer[..bytes_read]));

        let req_clone = request.clone();
        if request.contains("\r\n\r\n") && has_reached_request_end(req_clone) {
            break;
        }
    }

    Ok(request)
}

fn has_reached_request_end(request: String) -> bool {
    let content_length_key = "Content-Length: ".to_string();
    if let Some(cl_pos) = request.find(&content_length_key) {
        let after_cl = &request[(cl_pos + content_length_key.len())..];
        if let Some(line_break_pos) = after_cl.find("\r\n") {
            return has_reached_body_size(request.clone(), after_cl, line_break_pos);
        }
    } else {
        return true;
    }

    return false;
}

fn has_reached_body_size(request: String, after_cl: &str, line_break_pos: usize) -> bool {
    return match after_cl[..line_break_pos].trim().parse::<usize>() {
        Ok(content_length) => {
            let splitted: Vec<&str> = request.split("\r\n\r\n").collect();

            splitted.get(1).unwrap().len() >= content_length
        }
        Err(err) => {
            eprintln!("erro: {}", err);
            true
        }
    };
}

async fn handle_conn(mut stream: TcpStream, mut http_router: Router) {
    match read(&mut stream).await {
        Ok(req_str) => {
            let request = http::create_request(req_str);
            let response = http::create_response(stream);
            http_router.resolve_route(request, response).await;
        }
        Err(err) => eprintln!("error handling connection {}", err),
    }
}

#[derive(Clone)]
pub struct Router {
    root_dir: Option<String>,
    routes: HashMap<String, AsyncHandler>,
}

impl Router {
    pub fn add_route<F, Fut>(&mut self, verb: &str, path: &str, handler: F)
    where
        F: Fn(Request, Response) -> Fut + 'static + Send + Sync,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let mut route_key = String::new();
        route_key.push_str(verb);
        route_key.push_str(" ");
        route_key.push_str(path);
        self.routes.insert(
            route_key.clone(),
            Arc::new(move |req, res| Box::pin(handler(req, res))),
        );
        println!("added route {}", route_key);
    }

    pub async fn resolve_route(&mut self, mut request: Request, mut response: Response) {
        let mut found_route: Option<AsyncHandler> = None;
        let req_route = request.route();

        request.add_root_dir(self.root_dir.clone());

        for key in self.routes.keys() {
            let path_parts: Vec<&str> = req_route.split('/').collect();
            let key_parts: Vec<&str> = key.split('/').collect();

            if key_parts.len() != path_parts.len() {
                continue;
            }

            let mut found: bool = true;

            for (i, kp) in key_parts.iter().enumerate() {
                if kp.starts_with(":") {
                    request
                        .path_vars
                        .insert(kp.to_string(), path_parts.get(i).unwrap().to_string());
                    continue;
                }

                if kp != path_parts.get(i).unwrap() {
                    found = false;
                    break;
                }
            }

            if found {
                found_route = self.routes.get(key).cloned();
                break;
            }
        }

        if let Some(handler) = found_route {
            handler(request, response).await;
        } else {
            response.not_found(request).await;
        }
    }
}

pub fn new_router(root_dir: Option<String>) -> Router {
    let mut http_router = Router {
        root_dir,
        routes: HashMap::new(),
    };
    handler::add_handlers(&mut http_router);
    http_router
}
