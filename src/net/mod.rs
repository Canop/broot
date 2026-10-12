mod client;
mod message;
mod server;

pub use {
    client::Client,
    message::Message,
    server::Server,
};

/// Return the path of the server's socket file
#[cfg(unix)]
pub fn socket_address(server_name: &str) -> String {
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

/// Return the name of the server's pipe, relative to `\\.\pipe\`
#[cfg(windows)]
pub fn socket_address(server_name: &str) -> String {
    format!("broot-server-{server_name}")
}

pub fn random_server_name() -> String {
    use rand::distr::{
        Alphanumeric,
        SampleString,
    };
    Alphanumeric.sample_string(&mut rand::rng(), 10)
}
