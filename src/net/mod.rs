mod client;
mod message;
mod server;

pub use {
    client::Client,
    message::Message,
    server::Server,
};

pub fn socket_file_path(server_name: &str) -> String {
    #[cfg(target_os = "android")]
    {
        // On termux, /tmp is not writable and we're supposed
        // to use /data/data/com.termux/files/usr/tmp
        let usr_dir = "/data/data/com.termux/files/usr";
        if std::path::Path::new(usr_dir).is_dir() {
            return format!("{}/tmp/broot-server-{}.sock", usr_dir, server_name);
        }
        // maybe we're not in termux ? Fallback to /tmp
    }
    format!("/tmp/broot-server-{server_name}.sock")
}

pub fn random_server_name() -> String {
    use rand::{distributions::Alphanumeric, Rng};
    let name: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();
    name
}
