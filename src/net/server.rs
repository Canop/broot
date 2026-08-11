use {
    super::Message,
    crate::{
        command::Sequence,
        errors::NetError,
    },
    std::{
        fs,
        io::BufReader,
        os::unix::net::{
            UnixListener,
            UnixStream,
        },
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
            // A socket file is already present. It may belong to a live server
            // or be a stale leftover from a crashed instance. We probe it: when
            // a process is listening, connect succeeds and we must refuse to
            // overtake it (issue #1065); otherwise the file is stale and we
            // remove it before rebinding, exactly as before.
            if UnixStream::connect(&path).is_ok() {
                return Err(NetError::DuplicateServerName {
                    name: name.to_string(),
                });
            }
            match fs::remove_file(&path) {
                Ok(_) => {}
                Err(e) => return Err(NetError::Io { source: e }),
            }
        }
        let listener = UnixListener::bind(&path)?;
        info!("listening on {}", path);

        // we use only one thread as we don't want to support long connections
        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        let mut br = BufReader::new(&stream);
                        if let Some(sequence) = match Message::read(&mut br) {
                            Ok(Message::Command(command)) => {
                                info!("got single command {:?}", command);
                                // we convert it to a sequence
                                Some(Sequence::new_single(command))
                            }
                            Ok(Message::GetRoot) => {
                                debug!("got get root query");
                                let root = root.lock().unwrap();
                                let answer = Message::Root(root.to_string_lossy().to_string());
                                match answer.write(&mut stream) {
                                    Ok(()) => debug!("root path successfully returned"),
                                    Err(e) => warn!("error while answering: {:?}", e),
                                }
                                None
                            }
                            Ok(Message::Sequence(sequence)) => {
                                debug!("got sequence {sequence:?}");
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
        Ok(Self { path })
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        debug!("removing socket file");
        // The socket file may already be gone (taken over by another server, or
        // already cleaned up): never panic from Drop in that case.
        let _ = fs::remove_file(&self.path);
    }
}

#[cfg(test)]
mod test {
    use {
        super::Server,
        crate::command::Sequence,
        std::{
            path::PathBuf,
            sync::{
                Arc,
                Mutex,
            },
        },
        termimad::crossbeam::channel,
    };

    /// Two servers with the same name: the second must error instead of
    /// silently overtaking the first (issue #1065).
    #[test]
    fn second_server_with_same_name_errors() {
        let name = "broot-test-duplicate-server-name-do-not-use";
        let (tx, _rx) = channel::unbounded::<Sequence>();
        let root = Arc::new(Mutex::new(PathBuf::from("/")));
        let s1 = Server::new(name, tx.clone(), Arc::clone(&root)).expect("first server must bind");
        let second = Server::new(name, tx, root);
        assert!(
            second.is_err(),
            "second Server::new with the same name must error, not silently overtake"
        );
        drop(s1); // Drop removes the socket file for the unique test name
    }
}
