use {
    super::Message,
    crate::net::NetError,
    std::io::{
        BufReader,
        Read,
    },
};

#[cfg(unix)]
use std::os::unix::net::UnixStream;

#[cfg(windows)]
use interprocess::local_socket::{
    GenericNamespaced,
    Stream,
    prelude::*,
};

pub struct Client {
    /// Socket file path on unix, pipe name on Windows
    address: String,
}

impl Client {
    pub fn new(socket_name: &str) -> Self {
        Self {
            address: super::socket_address(socket_name),
        }
    }
    pub fn send(
        &self,
        message: &Message,
    ) -> Result<(), NetError> {
        debug!("try connecting {:?}", self.address);
        #[cfg(unix)]
        let mut stream = UnixStream::connect(&self.address)?;
        #[cfg(windows)]
        let mut stream = Stream::connect(self.address.as_str().to_ns_name::<GenericNamespaced>()?)?;
        message.write(&mut stream)?;
        if let Message::GetRoot = message {
            // we wait for the answer
            let mut br = BufReader::new(&stream);
            match Message::read(&mut br) {
                Ok(answer) => {
                    debug!("got an answer: {:?}", answer);
                    if let Message::Root(root) = answer {
                        println!("{root}");
                    }
                }
                Err(e) => {
                    warn!("got no answer but error {:?}", e);
                }
            }
        } else {
            // wait for the server to close the connection, so that the
            // message is known to be read before this process exits
            let _ = stream.read(&mut [0u8; 1]);
        }
        Ok(())
    }
}
