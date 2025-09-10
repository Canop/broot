use {
    super::Message,
    crate::{
        command::Sequence,
        errors::NetError,
    },
    std::{
        fs,
        io::BufReader,
        os::unix::net::UnixListener,
        path::PathBuf,
        sync::{
            Arc,
            Mutex,
        },
        thread,
    },
    termimad::crossbeam::channel::Sender,
};

pub struct Server {
    path: String,
}

impl Server {
    pub fn new(
        name: &str,
        tx: Sender<Sequence>,
        root: Arc<Mutex<PathBuf>>,
    ) -> Result<Self, NetError> {
        let path = super::socket_file_path(name);
        if fs::metadata(&path).is_ok() {
            match fs::remove_file(&path) {
                Ok(_) => {}
                Err(e) => {
                    return Err(NetError::Io {
                        source: e,
                    });
                }
            }
        }
        let listener = UnixListener::bind(&path)?;
        info!("listening on {}", &path);

        // we use only one thread as we don't want to support long connections
        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        let mut br = BufReader::new(&stream);
                        if let Some(sequence) = match Message::read(&mut br) {
                            Ok(Message::Command(command)) => {
                                info!("got single command {:?}", &command);
                                // we convert it to a sequence
                                Some(Sequence::new_single(command))
                            }
                            Ok(Message::GetRoot) => {
                                debug!("got get root query");
                                let root = root.lock().unwrap();
                                let answer =
                                    Message::Root(root.to_string_lossy().to_string());
                                match answer.write(&mut stream) {
                                    Ok(()) => debug!("root path successfully returned"),
                                    Err(e) => warn!("error while answering: {:?}", e),
                                }
                                None
                            }
                            Ok(Message::Sequence(sequence)) => {
                                debug!("got sequence {:?}", &sequence);
                                Some(sequence)
                            }
                            Ok(message) => {
                                debug!("got something not yet handled: {:?}", message);
                                None
                            }
                            Err(e) => {
                                warn!("Read error : {:?}", e);
                                None
                            }
                        } {
                            if let Err(e) = tx.send(sequence) {
                                warn!("error while sending {:?}", e);
                                return;
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Stream error : {:?}", e);
                    }
                }
            }
        });
        Ok(Self {
            path,
        })
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        debug!("removing socket file");
        fs::remove_file(&self.path).unwrap();
    }
}
