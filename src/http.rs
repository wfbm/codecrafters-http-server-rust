use std::collections::HashMap;
use std::io::Write;
use std::net::TcpStream;

use crate::encode;

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

pub struct Request {
    pub verb: String,
    pub path: String,
    pub root_dir: Option<String>,
    pub body: Option<String>,
    pub path_vars: HashMap<String, String>,
    pub headers: HashMap<String, String>,
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

    pub fn add_root_dir(&mut self, path: Option<String>) {
        self.root_dir = path;
    }
}

pub fn create_request(req_str: String) -> Request {
    let verb = extract_request_verb(req_str.clone());
    let path = extract_request_path(req_str.clone());
    let headers = extract_headers_from_request(req_str.clone());
    let body = extract_body_from_request(req_str.clone());

    Request {
        verb,
        path,
        headers,
        body,
        root_dir: None,
        path_vars: HashMap::new(),
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

fn extract_headers_from_request(req_str: String) -> HashMap<String, String> {
    let lines = req_str.split("\r\n");
    let mut headers: HashMap<String, String> = HashMap::new();

    for (i, line) in lines.enumerate() {
        if i == 0 {
            continue;
        }

        let mut parts = line.splitn(2, ':');
        let key = parts.next().unwrap();
        let value = parts.next().unwrap_or("");

        headers.insert(key.to_string(), value.trim().to_string());
    }

    headers
}

fn extract_body_from_request(req_str: String) -> Option<String> {
    let parts: Vec<&str> = req_str.split("\r\n\r\n").collect();
    let mut body: Option<String> = None;

    if parts.len() == 2 {
        let part = parts.get(1).unwrap().to_string();
        body = Some(part);
    }

    body
}

pub struct Response {
    conn: TcpStream,
    status_code: u32,
    status_description: String,
    headers: HashMap<String, String>,
    body: Option<String>,
}

impl Response {
    pub fn set_status_code(&mut self, status_code: u32) {
        self.status_code = status_code;
    }

    pub fn set_status_description(&mut self, status_description: String) {
        self.status_description = status_description;
    }

    pub fn set_header(&mut self, key: String, value: String) {
        self.headers.insert(key, value);
    }

    pub fn set_body(&mut self, body: Option<String>) {
        self.body = body;
    }

    pub fn ok(&mut self, request: Request, body: Option<String>) {
        self.set_status_code(200);
        self.set_status_description(String::from("OK"));
        self.set_body(body);
        self.flush(request);
    }

    pub fn no_content(&mut self, request: Request) {
        self.set_status_code(201);
        self.set_status_description(String::from("Created"));
        self.flush(request);
    }

    pub fn not_found(&mut self, request: Request) {
        self.set_status_code(404);
        self.set_status_description(String::from("Not Found"));
        self.flush(request);
    }

    pub fn internal_server_error(&mut self, request: Request, body: Option<String>) {
        self.set_status_code(500);
        self.set_status_description(String::from("Internal Server Error"));
        self.set_body(body);
        self.flush(request);
    }

    pub fn flush(&mut self, request: Request) {
        let mut response = String::new();
        response.push_str("HTTP/1.1 ");
        response.push_str(&self.status_code.to_string());
        response.push_str(" ");
        response.push_str(&self.status_description);
        response.push_str(&String::from("\r\n"));

        let mut body_content: Option<String> = None;

        if let Some(body) = &self.body {
            self.headers
                .insert(String::from("Content-Length"), body.len().to_string());
            body_content = Some(body.clone());
        }

        let body_content = self.encode_if_needed(body_content, request);

        for key in self.headers.keys() {
            response.push_str(key);
            response.push_str(": ");
            response.push_str(self.headers.get(key).unwrap());
            response.push_str("\r\n");
        }

        response.push_str("\r\n");
        response.push_str(&body_content.unwrap_or(String::from("")));

        let _ = self.conn.write_all(response.as_bytes());
    }

    fn encode_if_needed(
        &mut self,
        mut content: Option<String>,
        request: Request,
    ) -> Option<String> {
        let accept_encoding = request.headers.get("Accept-Encoding");

        if let Some(encoding_list) = accept_encoding {
            let encoding_options = encoding_list.split(",");

            for encoding in encoding_options {
                match encode::new_encoder(encoding.trim()) {
                    Ok(encoder) => {
                        self.headers
                            .insert(String::from("Content-Encoding"), String::from("gzip"));
                        content = Some(encoder.encode(content.unwrap_or("".to_string())));
                        break;
                    }
                    Err(err) => {
                        eprintln!("{err}");
                    }
                }
            }
        }

        content
    }
}

pub fn create_response(stream: TcpStream) -> Response {
    Response {
        conn: stream,
        status_code: 200,
        status_description: String::from("OK"),
        headers: HashMap::new(),
        body: None,
    }
}
