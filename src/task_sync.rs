use {
    crossbeam::channel::{self, bounded, select, Receiver},
    std::thread,
    termimad::TimedEvent,
};

pub enum Either<A, B> {
    First(A),
    Second(B),
}

#[derive(Debug, Clone)]
pub enum ComputationResult<V> {
    NotComputed, // not computed but will probably be
    Done(V),
    None, // nothing to compute, cancelled, failed, etc.
}
impl<V> ComputationResult<V> {
    pub fn is_done(&self) -> bool {
        matches!(&self, Self::Done(_))
    }
    pub fn is_not_computed(&self) -> bool {
        matches!(&self, Self::NotComputed)
    }
    pub fn is_some(&self) -> bool {
        !matches!(&self, Self::None)
    }
    pub fn is_none(&self) -> bool {
        matches!(&self, Self::None)
    }
}

/// The dam controls the flow of events.
/// A dam is used in broot to manage long computations and,
/// when the user presses a key, either tell the computation
/// to stop (the computation function checking `has_event`)
/// or drop the computation.
pub struct Dam {
    receiver: Receiver<TimedEvent>,
    in_dam: Option<TimedEvent>,
}

impl Dam {
    pub fn from(receiver: Receiver<TimedEvent>) -> Self {
        Self {
            receiver,
            in_dam: None,
        }
    }
    pub fn unlimited() -> Self {
        Self::from(channel::never())
    }

    /// provide an observer which can be used for periodic
    /// check a task can be used.
    /// The observer can safely be moved to another thread
    /// but Be careful not to use it
    /// after the event listener started again. In any case
    /// using try_compute should be preferred for immediate
    /// return to the ui thread.

    pub fn observer(&self) -> DamObserver {
        DamObserver::from(self)
    }

    /// launch the computation on a new thread and return
    /// when it finishes or when a new event appears on
    /// the channel.
    /// Note that the task itself isn't interrupted so that
    /// this should not be used when many tasks are expected
    /// to be launched (or it would result in many working
    /// threads uselessly working in the background) : use
    /// dam.has_event from inside the task whenever possible.
    pub fn try_compute<V: Send + 'static, F: Send + 'static + FnOnce() -> ComputationResult<V>>(
        &mut self,
        f: F,
    ) -> ComputationResult<V> {
        let (comp_sender, comp_receiver) = bounded(1);
        thread::spawn(move || {
            let comp_res = time!("comp in dam", f());
            if comp_sender.send(comp_res).is_err() {
                debug!("no channel at end of computation");
            }
        });
        self.select(comp_receiver)
    }

    pub fn select<V>(
        &mut self,
        comp_receiver: Receiver<ComputationResult<V>>,
    ) -> ComputationResult<V> {
        if self.in_dam.is_some() {
            // should probably not happen
            debug!("There's already an event in dam");
            ComputationResult::None
        } else {
            select! {
                recv(self.receiver) -> event => {
                    // interruption
                    debug!("dam interrupts computation");
                    self.in_dam = event.ok();
                    ComputationResult::None
                }
                recv(comp_receiver) -> comp_res => {
                    // computation finished
                    comp_res.unwrap_or(ComputationResult::None)
                }
            }
        }
    }

    /// non blocking
    pub fn has_event(&self) -> bool {
        !self.receiver.is_empty()
    }

    /// drop all events, returns the count of removed events
    pub fn clear(&mut self) -> usize {
        let mut n = 0;
        while self.has_event() {
            n += 1;
            self.next_event();
        }
        n
    }

    /// block until next event (including the one which
    ///  may have been pushed back into the dam).
    /// no event means the source is dead (i.e. we
    /// must quit broot)
    /// There's no event kept in dam after this call.
    pub fn next_event(&mut self) -> Option<TimedEvent> {
        if self.in_dam.is_some() {
            self.in_dam.take()
        } else {
            match self.receiver.recv() {
                Ok(event) => Some(event),
                Err(_) => {
                    debug!("dead dam"); // should be logged once
                    None
                }
            }
        }
    }

    // or maybed return either Option<TimedEvent> or Option<T> ?
    pub fn next<T>(&mut self, other: &Receiver<T>) -> Either<Option<TimedEvent>, Option<T>> {
        if self.in_dam.is_some() {
            Either::First(self.in_dam.take())
        } else {
            select! {
                recv(self.receiver) -> event => Either::First(match event {
                    Ok(event) => Some(event),
                    Err(_) => {
                        debug!("dead dam"); // should be logged once
                        None
                    }
                }),
                recv(other) -> o => Either::Second(match o {
                    Ok(o) => Some(o),
                    Err(_) => {
                        debug!("dead other");
                        None
                    }
                }),
            }
        }
    }
}

pub struct DamObserver {
    receiver: Receiver<TimedEvent>,
}
impl DamObserver {
    pub fn from(dam: &Dam) -> Self {
        Self {
            receiver: dam.receiver.clone(),
        }
    }
    /// be careful that this can be used as a thread
    /// stop condition only before the event receiver
    /// start being active to avoid a race condition.
    pub fn has_event(&self) -> bool {
        !self.receiver.is_empty()
    }
}

/// wraps either a computation in progress, or a finished
/// one (even a failed or useless one).
/// This can be stored in a map to avoid starting computations
/// more than once.
#[derive(Debug, Clone)]
pub enum Computation<V> {
    InProgress(Receiver<ComputationResult<V>>),
    Finished(ComputationResult<V>),
}
