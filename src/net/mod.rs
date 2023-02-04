mod client;
mod message;
mod server;

pub use {
    client::Client,
    message::Message,
    server::Server,
};

pub fn socket_file_path(server_name: &str) -> String {
    format!("/tmp/broot-server-{server_name}.sock")
}
