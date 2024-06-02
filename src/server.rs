use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::net::{TcpListener, TcpStream};

use crate::handler;
use crate::http;
use crate::http::{Request, Response};

pub struct Server {
    router: Router,
}

impl Server {
    pub fn start(&self) {
        let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
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

pub fn new_server(directory: String) -> Server {
    let mut dir: Option<String> = None;

    if !directory.is_empty() {
        dir = Some(directory);
    }

    let http_router = new_router(dir.clone());

    Server {
        router: http_router,
    }
}

fn read(mut stream: &TcpStream) -> String {
    let mut buffer = [0; 512];
    let mut request = String::new();

    loop {
        let bytes_read = stream.read(&mut buffer).unwrap();
        if bytes_read == 0 {
            break;
        }

        request.push_str(&String::from_utf8_lossy(&buffer[..bytes_read]));

        if request.contains("\r\n\r\n") {
            break;
        }
    }

    request
}

async fn handle_conn(stream: TcpStream, mut http_router: Router) {
    let req_str = read(&stream);
    let request = http::create_request(req_str);
    let response = http::create_response(stream);

    http_router.resolve_route(request, response);
}

#[derive(Clone)]
pub struct Router {
    root_dir: Option<String>,
    routes: HashMap<String, fn(Request, Response)>,
}

impl Router {
    pub fn add_route(&mut self, verb: &str, path: &str, handler: fn(Request, Response)) {
        let mut route_key = String::new();
        route_key.push_str(verb);
        route_key.push_str(" ");
        route_key.push_str(path);
        self.routes.insert(route_key.clone(), handler);
        println!("added route {}", route_key);
    }

    pub fn resolve_route(&mut self, mut request: Request, response: Response) {
        let mut found_route: Option<fn(Request, Response)> = None;
        let req_route = request.route();

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
                found_route = self.routes.get(key).copied();
                break;
            }
        }

        if let Some(handler) = found_route {
            handler(request, response);
        } else {
            self.resolve_file(request, response);
        }
    }

    fn resolve_file(&self, request: Request, mut response: Response) {
        let mut path = String::new();
        if let Some(request_path) = &self.root_dir {
            path.push_str(&request_path);
            path.push_str(&request.path);

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
}

pub fn new_router(root_dir: Option<String>) -> Router {
    let mut http_router = Router {
        root_dir,
        routes: HashMap::new(),
    };
    handler::add_handlers(&mut http_router);
    http_router
}
