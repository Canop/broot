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
    serde::Deserialize,
    std::{
        path::PathBuf,
        thread,
    },
    termimad::crossbeam::channel,
};

const DEBOUNCE_MAX_DELAY: std::time::Duration = std::time::Duration::from_millis(500);

#[derive(Debug, Clone, Copy, Default, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WatchStrategy {
    /// Choose the best (may be plaform dependent in the future, today is never-poll)
    #[default]
    Auto,
    /// Try to use inotify, gives up if fails (due to platform or number of watches limit)
    #[serde(alias = "never_poll")]
    NeverPoll,
    /// Try to use inotify, fallback to polling if not available
    #[serde(alias = "allow_poll")]
    AllowPoll,
}

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
    watched: Option<PathBuf>,
    strategy: WatchStrategy,
}

impl Watcher {
    pub fn new(
        tx_seqs: channel::Sender<Sequence>,
        strategy: WatchStrategy,
    ) -> Self {
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
                        debug!("notify sender disconnected, stopping notify watcher thread");
                        break;
                    }
                }
            }
        });
        Self {
            notify_sender,
            strategy,
            notify_watcher: None,
            watched: None,
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
        path: PathBuf,
    ) -> Result<(), ProgramError> {
        debug!("start watching path {:?}", path);
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
                .insert(
                    Self::make_notify_watcher(
                        self.notify_sender.clone(),
                        self.strategy,
                    )?
                ),
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
    fn make_notify_watcher(
        sender: channel::Sender<()>,
        watch_strategy: WatchStrategy,
    ) -> Result<RecommendedWatcher, ProgramError> {
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
                        debug!("error when notifying on notify event: {}", e);
                    }
                }
                Err(e) => warn!("watch error: {:?}", e),
            })?;
        let mut config = notify::Config::default()
            .with_compare_contents(false)
            .with_follow_symlinks(false);
        info!("watch_strategy: {:?}", watch_strategy);
        if watch_strategy != WatchStrategy::AllowPoll {
            debug!("watcher polling is disabled");
            config = config.with_manual_polling();
        }
        notify_watcher.configure(config)?;
        Ok(notify_watcher)
    }
    pub fn stop_watching(&mut self) -> Result<(), ProgramError> {
        if let Some(path) = self.watched.take() {
            if let Some(nw) = self.notify_watcher.as_mut() {
                debug!("stop watching path {:?}", path);
                if let Err(e) = nw.unwatch(&path) {
                    warn!("error when unwatching path {:?}: {}", path, e);
                }
            }
        }
        Ok(())
    }
}
