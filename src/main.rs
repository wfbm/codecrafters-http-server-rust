// Uncomment this block to pass the first stage
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::str;

const VALID_ENDPOINTS: &'static [&'static str] = &["/"];

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let request = read(&stream);
                let valid = validate_endpoint(request);

                if valid {
                    let _ = stream.write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes());
                } else {
                    let _ = stream.write_all("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes());
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
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

fn validate_endpoint(request: String) -> bool {
    let mut lines = request.split("\r\n");

    if let Some(request_line) = lines.next() {
        let mut request_parts = request_line.split(" ");
        let mut path = request_parts.find(|&p| p.starts_with("/")).unwrap_or("");

        return VALID_ENDPOINTS.contains(&mut path);
    }

    return false;
}
