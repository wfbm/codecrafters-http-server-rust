use std::collections::HashMap;
use std::net::TcpStream;

pub const GET: &str = "GET";
#[allow(dead_code)]
pub const POST: &str = "POST";
#[allow(dead_code)]
pub const PUT: &str = "PUT";
#[allow(dead_code)]
pub const PATCH: &str = "PATCH";
#[allow(dead_code)]
pub const DELETE: &str = "DELETE";
#[allow(dead_code)]
pub const OPTIONS: &str = "OPTIONS";

const VALID_VERBS: &'static [&'static str] = &["GET", "POST", "PUT", "PATCH", "DELETE", "OPTIONS"];

pub const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\n\r\n";
pub const NOT_FOUND_RESPONSE: &str = "HTTP/1.1 404 Not Found\r\n\r\n";

pub struct Request {
    pub verb: String,
    pub path: String,
    pub path_vars: HashMap<String, String>,
    pub header: HashMap<String, String>,
}

impl Request {
    pub fn route(&mut self) -> String {
        let mut route = String::new();
        route.push_str(&self.verb);
        route.push_str(" ");
        route.push_str(&self.path);
        println!("{route}");
        route
    }
}

pub fn create_request(req_str: String) -> Request {
    let verb = extract_request_verb(req_str.clone());
    let path = extract_request_path(req_str.clone());

    Request {
        verb,
        path,
        path_vars: HashMap::new(),
        header: HashMap::new(),
    }
}

fn extract_request_verb(req_str: String) -> String {
    extract_from_request(req_str, |p| VALID_VERBS.contains(&mut &p))
}

fn extract_request_path(req_str: String) -> String {
    extract_from_request(req_str, |p| p.starts_with("/"))
}

fn extract_from_request<F>(req_str: String, mut filter: F) -> String
where
    F: FnMut(&str) -> bool,
{
    let mut path = String::new();
    let mut lines = req_str.split("\r\n");

    if let Some(request_line) = lines.next() {
        let mut request_parts = request_line.split(" ");
        path = request_parts.find(|p| filter(p)).unwrap_or("").to_string();
    }

    return path;
}

pub struct Response {
    pub conn: TcpStream,
}

pub fn create_response(stream: TcpStream) -> Response {
    Response { conn: stream }
}
