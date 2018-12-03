use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicUsize, Ordering};


pub struct TaskLifetime {
    initial_value: usize,
    external_value: Arc<AtomicUsize>,
}

impl TaskLifetime {
    pub fn new(external_value: &Arc<AtomicUsize>) -> TaskLifetime {
        TaskLifetime {
            initial_value: external_value.load(Ordering::Relaxed),
            external_value: Arc::clone(external_value),
        }
    }
    pub fn unlimited() -> TaskLifetime {
        TaskLifetime {
            initial_value: 0,
            external_value: Arc::new(AtomicUsize::new(0)),
        }
    }
    pub fn is_expired(&self) -> bool {
        self.initial_value != self.external_value.load(Ordering::Relaxed)
    }
}
