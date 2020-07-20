
use {
    super::{
        Message,
    },
    crate::{
        errors::NetError,
    },
    std::{
        os::unix::net::{
            UnixStream,
        },
    },
};

pub struct Client {
    path: String,
}

impl Client {
    pub fn new(socket_name: &str) -> Self {
        Self {
            path: super::socket_file_path(socket_name),
        }
    }
    pub fn send(&self, message: &Message) -> Result<(), NetError> {
        debug!("try connecting {:?}", &self.path);
        let mut stream = UnixStream::connect(&self.path)?;
        message.write(&mut stream)?;
        Ok(())
    }
}
