use lazy_static::lazy_static;
use parking_lot::ReentrantMutex;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use std::{panic, thread};

//This lock will not be contended in happy path - only when an actor panics!
lazy_static! {
    static ref PANICS: Arc<ReentrantMutex<RefCell<HashMap<u64, u8>>>> =
        Arc::new(ReentrantMutex::new(RefCell::new(HashMap::new())));
}
static PANIC_TOLERANCE: u8 = 3;

thread_local! {
   static  ACTOR_ID: RefCell<u64> = RefCell::new(0);
}

pub(super) struct PanicWatch;

impl PanicWatch {
    pub(super) fn new() -> Self {
        //Set panic handler for for the actors. We don't want to eject actors on the very
        //first instance that it panics. Panics may be due to corrupt messages.
        //Hence we maintain a tolerance limit.
        panic::set_hook(Box::new(|_panic_info| {
            if thread::panicking() {
                ACTOR_ID.with(|id| {
                    let lock = PANICS.lock();
                    let mut panics = lock.borrow_mut();
                    match panics.get_mut(&id.borrow()) {
                        Some(count) => *count += 1,
                        None => {
                            panics.insert(*id.borrow(), 1);
                        }
                    }
                });
            }
        }));
        Self
    }

    pub(super) fn tolerance() -> u8 {
        PANIC_TOLERANCE
    }

    pub(super) fn set_watch(actor_id: u64) {
        ACTOR_ID.with(|id| {
            *id.borrow_mut() = actor_id;
        });
    }

    pub(super) fn remove_watch(actor_id: &u64) {
        let lock = PANICS.lock();
        let mut panics = lock.borrow_mut();
        panics.remove(actor_id);
    }

    pub(super) fn has_exceeded_tolerance(actor_id: u64) -> bool {
        let lock = PANICS.lock();
        let panics = lock.borrow();
        match panics.get(&actor_id) {
            Some(count) => *count >= PANIC_TOLERANCE,
            None => false,
        }
    }
    pub(super) fn count(actor_id: u64) -> u8 {
        let lock = PANICS.lock();
        let panics = lock.borrow();
        match panics.get(&actor_id) {
            Some(count) => *count,
            None => 0,
        }
    }
}
