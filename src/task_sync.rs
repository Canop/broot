
use {
    crossbeam::channel::{
        self,
        bounded,
        Receiver,
    },
    lazy_static::lazy_static,
    std::{
        thread,
    },
    termimad::Event,
};

#[derive(Debug, Clone)]
pub enum ComputationResult<T> {
    NotComputed, // not computed but will probably be
    Done(T),
    None, // nothing to compute, cancelled, failed, etc.
}
impl<T> ComputationResult<T> {
    pub fn is_done(&self) -> bool {
        match &self {
            Self::Done(_) => true,
            _ => false,
        }
    }
    pub fn is_not_computed(&self) -> bool {
        match &self {
            Self::NotComputed => true,
            _ => false,
        }
    }
    pub fn is_none(&self) -> bool {
        match &self {
            Self::None => true,
            _ => false,
        }
    }
}

/// a dam is used in broot to manage long computations and,
/// when the user presses a key, either tell the computation
/// to stop (the computation function checking `has_event`)
/// or drop the computation.
pub struct Dam {
    receiver: Receiver<Event>,
    in_dam: Option<Event>,
}

// cache problem: we must receive the result of dropped
// computattion to use it if we receive the same
// computation key later
impl Dam {
    pub fn from(receiver: Receiver<Event>) -> Self {
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
    /// using try_compute should be prefered for immediate
    /// return to the ui thread.

    pub fn observer(&self) -> DamObserver {
        DamObserver::from(self)
    }

    /// launch the computation on a new thread and return
    /// when it finishes or when a new event appears on
    /// the channel
    pub fn try_compute<T: Send + 'static, F: Send + 'static + FnOnce() -> ComputationResult<T>>(
        &mut self,
        f: F,
    ) -> ComputationResult<T> {
        let (comp_sender, comp_receiver) = bounded(1);
        thread::spawn(move|| {
            let comp_res = time!(Debug, "comp in dam", f());
            if comp_sender.send(comp_res).is_err() {
                debug!("no channel at end of computation");
            }
        });
        self.select(comp_receiver)
    }

    fn select<T>(
        &mut self,
        computation_channel: Receiver<ComputationResult<T>>,
    ) -> ComputationResult<T> {
        if self.in_dam.is_some() {
            // should probably not happen
            debug!("There's already an event in dam");
            ComputationResult::None
        } else {
            //
            debug!("start select! in dam");
            select! {
                recv(self.receiver) -> event => {
                    // interruption
                    debug!("dam interrupts computation");
                    self.in_dam = event.ok();
                    ComputationResult::None
                }
                recv(computation_channel) -> comp_res => {
                    // computation finished
                    debug!("computation passes dam");
                    comp_res.unwrap_or(ComputationResult::None)
                }
            }
        }
    }

    /// non blocking
    pub fn has_event(&self) -> bool {
        !self.receiver.is_empty()
    }

    /// block until next event (including the one which
    ///  may have been pushed back into the dam).
    /// no event means the source is dead (i.e. we
    /// must quit broot)
    /// There's no event kept in dam after this call.
    pub fn next_event(&mut self) -> Option<Event> {
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
}

pub struct DamObserver {
    receiver: Receiver<Event>,
}
impl DamObserver {
    pub fn from(dam: &Dam) -> Self {
        Self {
            receiver: dam.receiver.clone()
        }
    }
    /// be careful that this can be used as a thread
    /// stop condition only before the event receiver
    /// start being active to avoid a race condition.
    pub fn has_event(&self) -> bool {
        !self.receiver.is_empty()
    }
}

