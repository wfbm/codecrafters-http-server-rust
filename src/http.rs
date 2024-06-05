use std::collections::HashMap;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use crate::encode;

pub const GET: &str = "GET";
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

    pub async fn ok(&mut self, request: Request, body: Option<String>) {
        self.set_status_code(200);
        self.set_status_description(String::from("OK"));
        self.set_body(body);
        self.flush(request).await;
    }

    pub async fn no_content(&mut self, request: Request) {
        self.set_status_code(201);
        self.set_status_description(String::from("Created"));
        self.flush(request).await;
    }

    pub async fn not_found(&mut self, request: Request) {
        self.set_status_code(404);
        self.set_status_description(String::from("Not Found"));
        self.flush(request).await;
    }

    pub async fn internal_server_error(&mut self, request: Request, body: Option<String>) {
        self.set_status_code(500);
        self.set_status_description(String::from("Internal Server Error"));
        self.set_body(body);
        self.flush(request).await;
    }

    pub async fn flush(&mut self, request: Request) {
        let response_line = self.create_response_line();
        let body_content = self.create_response_body(request);
        let headers = self.create_response_headers();

        let mut response = String::new();
        response.push_str(&response_line);
        response.push_str(&headers);

        let mut response_buffer = response.as_bytes().to_vec();

        if let Some(body_response) = body_content {
            response_buffer.extend_from_slice(&body_response);
        }

        let _ = self.conn.write_all(&response_buffer).await;
    }

    fn create_response_line(&self) -> String {
        let mut response = String::new();
        response.push_str("HTTP/1.1 ");
        response.push_str(&self.status_code.to_string());
        response.push_str(" ");
        response.push_str(&self.status_description);
        response.push_str(&String::from("\r\n"));
        response
    }

    fn create_response_body(&mut self, request: Request) -> Option<Vec<u8>> {
        let mut body_content: Option<Vec<u8>> = None;

        if let Some(body) = &self.body {
            body_content = self.encode_body_if_needed(body.clone(), request);
            self.headers.insert(
                String::from("Content-Length"),
                body_content.clone().unwrap().len().to_string(),
            );
        }

        body_content
    }

    fn create_response_headers(&self) -> String {
        let mut headers = String::new();

        for key in self.headers.keys() {
            headers.push_str(key);
            headers.push_str(": ");
            headers.push_str(self.headers.get(key).unwrap());
            headers.push_str("\r\n");
        }

        headers.push_str("\r\n");
        headers
    }

    fn encode_body_if_needed(&mut self, content: String, request: Request) -> Option<Vec<u8>> {
        if content.is_empty() {
            return None;
        }

        let accept_encoding = request.headers.get("Accept-Encoding");
        let buffer: Vec<u8> = content.as_bytes().to_vec();

        if let Some(encoding_list) = accept_encoding {
            let encoding_options = encoding_list.split(",");

            for encoding in encoding_options {
                match encode::new_encoder(encoding.trim()) {
                    Ok(encoder) => {
                        self.headers.insert(
                            String::from("Content-Encoding"),
                            encoding.trim().to_string(),
                        );
                        return Some(encoder.encode(content));
                    }
                    Err(err) => {
                        eprintln!("{err}");
                    }
                }
            }
        }

        Some(buffer)
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
