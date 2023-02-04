
use {
    super::{
        Message,
    },
    crate::{
        errors::NetError,
    },
    std::{
        io::BufReader,
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
        if let Message::GetRoot = message {
            // we wait for the answer
            let mut br = BufReader::new(&stream);
            match Message::read(&mut br) {
                Ok(answer) => {
                    debug!("got an answer: {:?}", &answer);
                    if let Message::Root(root) = answer {
                        println!("{root}");
                    }
                }
                Err(e) => {
                    warn!("got no answer but error {:?}", e);
                }
            }
        }
        Ok(())
    }
}
