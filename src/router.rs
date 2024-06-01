use std::collections::HashMap;

use crate::handler;
use crate::http::{Request, Response};

pub struct Router {
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

            if key_parts.get(0) != path_parts.get(0) {
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
            handler::handle_not_found(response);
        }
    }
}

pub fn new_router() -> Router {
    let mut http_router = Router {
        routes: HashMap::new(),
    };
    handler::add_handlers(&mut http_router);
    http_router
}
