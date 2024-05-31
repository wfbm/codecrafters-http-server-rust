use std::io::Read;
use std::net::{TcpListener, TcpStream};

mod handler;
mod http;
mod router;

fn main() {
    let mut http_router = router::new_router();
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let req_str = read(&stream);
                let request = http::create_request(req_str);
                let response = http::create_response(stream);

                http_router.resolve_route(request, response);
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
