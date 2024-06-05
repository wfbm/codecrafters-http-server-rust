use clap::{arg, Command};

mod encode;
mod handler;
mod http;
mod server;

#[tokio::main]
async fn main() {
    let matches = Command::new("server")
        .version("1.0.0")
        .about("http server")
        .arg(arg!(--directory <DIR>))
        .get_matches();

    let empty_dir = String::new();
    let dir_arg = matches.get_one::<String>("directory").unwrap_or(&empty_dir);

    let server = server::new_server(dir_arg.to_string());
    server.start().await;
}
