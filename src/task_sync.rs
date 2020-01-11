use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use lazy_static::lazy_static;

/// a TL initialized from an Arc<AtomicUsize> stays
///  alive as long as the passed arc doesn't change.
/// When it changes, is_expired returns true
#[derive(Debug, Clone)]
pub struct TaskLifetime {
    initial_value: usize,
    external_value: Arc<AtomicUsize>,
}

impl TaskLifetime {
    pub fn new(external_value: Arc<AtomicUsize>) -> TaskLifetime {
        TaskLifetime {
            initial_value: external_value.load(Ordering::Relaxed),
            external_value,
        }
    }
    pub fn unlimited() -> TaskLifetime {
        // Use a global static Arc<AtomicUsize> so that we don't have to
        // allocate more than once
        lazy_static! {
            static ref ZERO: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
        }

        TaskLifetime {
            initial_value: 0,
            external_value: ZERO.clone(),
        }
    }
    pub fn is_expired(&self) -> bool {
        self.initial_value != self.external_value.load(Ordering::Relaxed)
    }
}
