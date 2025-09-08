use {
    crate::{
        command::Sequence,
        errors::ProgramError,
    },
    notify::{
        RecommendedWatcher,
        RecursiveMode,
        Watcher as NotifyWatcher,
        event::{
            AccessKind,
            AccessMode,
            DataChange,
            EventKind,
            ModifyKind,
        },
    },
    std::path::PathBuf,
    termimad::crossbeam::channel::Sender,
};

/// Watch for notify events on a path, and send a :refresh sequence when a change is detected
pub struct Watcher {
    tx_seqs: Sender<Sequence>,
    notify_watcher: Option<RecommendedWatcher>,
    watched: Option<PathBuf>,
}

impl Watcher {
    pub fn new(tx_seqs: Sender<Sequence>) -> Self {
        Self {
            tx_seqs,
            notify_watcher: None,
            watched: None,
        }
    }
    /// stop watching the previous path, watch new one.
    ///
    /// In case of error, we try to stop watching the previous path anyway.
    pub fn watch(
        &mut self,
        path: PathBuf,
    ) -> Result<(), ProgramError> {
        let notify_watcher = match self.notify_watcher.as_mut() {
            Some(nw) => {
                if let Some(path) = self.watched.take() {
                    debug!("stop watching previous path {:?}", path);
                    nw.unwatch(&path)?;
                }
                nw
            }
            None => self
                .notify_watcher
                .insert(Self::make_notify_watcher(self.tx_seqs.clone())?),
        };
        if !path.exists() {
            warn!("watch path doesn't exist: {:?}", path);
            return Ok(());
        }
        let res = if path.is_dir() {
            debug!("add watch dir {:?}", &path);
            notify_watcher.watch(&path, RecursiveMode::Recursive)
        } else if path.is_file() {
            debug!("add watch file {:?}", &path);
            notify_watcher.watch(&path, RecursiveMode::NonRecursive)
        } else {
            warn!("watch path is neither file nor directory: {:?}", path);
            Ok(())
        };
        match &res {
            Ok(()) => {
                self.watched = Some(path);
            }
            Err(_) => {
                // the RecommendedWatcher sometimes ends in an unconsistent state when failing
                // to watch a path, so we drop it
                self.notify_watcher = None;
            }
        }
        Ok(res?)
    }
    fn make_notify_watcher(tx_seqs: Sender<Sequence>) -> Result<RecommendedWatcher, ProgramError> {
        let mut notify_watcher =
            notify::recommended_watcher(move |res: notify::Result<notify::Event>| match res {
                Ok(we) => {
                    // Warning: don't log we, or a Modify::Any event, as this could cause infinite
                    // loop while the logger writes to the file being watched (if the log file is
                    // inside the watched directory)
                    match we.kind {
                        EventKind::Modify(ModifyKind::Metadata(_)) => {
                            debug!("ignoring metadata change");
                            return; // useless event
                        }
                        EventKind::Modify(ModifyKind::Data(DataChange::Any)) => {
                            // might be data append, we prefer to ignore it
                            // as some cases (eg log files) are very noisy
                            return;
                        }
                        EventKind::Access(AccessKind::Close(AccessMode::Write)) => {
                            debug!("close write event: {we:?}");
                        }
                        EventKind::Access(_) => {
                            // we don't want to watch for reads
                            return;
                        }
                        _ => {
                            debug!("notify event: {we:?}");
                        }
                    }
                    let sequence = Sequence::new_single(":refresh");
                    if let Err(e) = tx_seqs.send(sequence) {
                        debug!("error when notifying on notify event: {}", e);
                    }
                }
                Err(e) => warn!("watch error: {:?}", e),
            })?;
        notify_watcher.configure(
            notify::Config::default()
                .with_compare_contents(false)
                .with_follow_symlinks(false),
        )?;
        Ok(notify_watcher)
    }
    pub fn stop_watching(&mut self) -> Result<(), ProgramError> {
        if let Some(path) = self.watched.take() {
            if let Some(nw) = self.notify_watcher.as_mut() {
                debug!("stop watching path {:?}", path);
                nw.unwatch(&path)?;
            }
        }
        Ok(())
    }
}
