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
    std::{
        path::PathBuf,
        thread,
    },
    termimad::crossbeam::channel,
};

const DEBOUNCE_MAX_DELAY: std::time::Duration = std::time::Duration::from_millis(500);

/// Watch for notify events on a path, and send a :refresh sequence when a change is detected
///
/// inotify events are debounced:
/// - an isolated event sends a refresh immediately
/// - successive events after the first one will have to wait a little
/// - there's at most one refrest sent every DEBOUNCE_MAX_DELAY
/// - if there's a long sequence of events, it's guaranteed that there's one
///   refresh sent every DEBOUNCE_MAX_DELAY
/// - the last event of the sequence is always sent (with a delay of
///   at most DEBOUNCE_MAX_DELAY), ensuring we don't miss any change
pub struct Watcher {
    notify_sender: channel::Sender<()>,
    notify_watcher: Option<RecommendedWatcher>,
    watched: Vec<PathBuf>,
}

impl Watcher {
    pub fn new(tx_seqs: channel::Sender<Sequence>) -> Self {
        let (notify_sender, notify_receiver) = channel::unbounded();
        thread::spawn(move || {
            let mut period_events = 0;
            loop {
                match notify_receiver.recv_timeout(DEBOUNCE_MAX_DELAY) {
                    Ok(()) => {
                        period_events += 1;
                        if period_events > 1 {
                            continue;
                        }
                        debug!("sending single event");
                        Self::send_refresh(&tx_seqs);
                    }
                    Err(channel::RecvTimeoutError::Timeout) => {
                        if period_events <= 1 {
                            continue;
                        }
                        debug!("sending aggregation of {} pending events", period_events - 1);
                        Self::send_refresh(&tx_seqs);
                        period_events = 0;
                    }
                    Err(channel::RecvTimeoutError::Disconnected) => {
                        info!("notify sender disconnected, stopping notify watcher thread");
                        break;
                    }
                }
            }
        });
        Self {
            notify_sender,
            notify_watcher: None,
            watched: Default::default(),
        }
    }
    fn send_refresh(
        tx_seqs: &channel::Sender<Sequence>,
    ) {
        if !tx_seqs.is_empty() {
            // let's avoid accumulating refreshes when the tree is long to update
            debug!("skipping refresh, channel full");
            return;
        }
        let sequence = Sequence::new_single(":refresh");
        if let Err(e) = tx_seqs.send(sequence) {
            warn!("error when sending sequence from watcher: {}", e);
        }
    }
    /// stop watching the previous path, watch new one.
    ///
    /// In case of error, we try to stop watching the previous path anyway.
    pub fn watch(
        &mut self,
        paths: Vec<PathBuf>,
    ) -> Result<(), ProgramError> {
        debug!("start watching new paths");
        let notify_watcher = match self.notify_watcher.as_mut() {
            Some(nw) => {
                for path in self.watched.drain(..) {
                    debug!("stop watching previous path {:?}", path);
                    if let Err(e) = nw.unwatch(&path) {
                        warn!("error when unwatching path {:?}: {}", path, e);
                    }
                }
                nw
            }
            None => self
                .notify_watcher
                .insert(Self::make_notify_watcher(self.notify_sender.clone())?),
        };
        let mut err = None;
        for path in &paths {
            if !path.exists() {
                warn!("watch path doesn't exist: {:?}", path);
                return Ok(());
            }
            debug!("add watch {:?}", &path);
            if let Err(e) = notify_watcher.watch(path, RecursiveMode::NonRecursive) {
                warn!("error when watching path {:?}: {}", path, e);
                err = Some(e);
                break;
            }
        }
        if let Some(err) = err {
            // the RecommendedWatcher sometimes ends in an unconsistent state when failing
            // to watch a path, so we drop it
            self.notify_watcher = None;
            Err(err.into())
        } else {
            self.watched = paths;
            Ok(())
        }
    }
    fn make_notify_watcher(sender: channel::Sender<()>) -> Result<RecommendedWatcher, ProgramError> {
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
                    if let Err(e) = sender.send(()) {
                        info!("error when notifying on notify event: {}", e);
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
        for path in self.watched.drain(..) {
            if let Some(nw) = self.notify_watcher.as_mut() {
                debug!("stop watching previous path {:?}", path);
                if let Err(e) = nw.unwatch(&path) {
                    warn!("error when unwatching path {:?}: {}", path, e);
                }
            }
        }
        Ok(())
    }
}
