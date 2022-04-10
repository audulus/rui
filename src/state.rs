use std::any::Any;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use tao::event_loop::EventLoopProxy;
use std::cell::RefCell;
use crate::*;

static STATE_DIRTY: AtomicBool = AtomicBool::new(false);

thread_local! {
    pub static ENABLE_DIRTY: RefCell<bool> = RefCell::new(true);
}

pub(crate) fn is_state_dirty() -> bool {
    STATE_DIRTY.load(Ordering::Relaxed)
}

pub(crate) fn set_state_dirty() {
    ENABLE_DIRTY.with(|enable| {
        if *enable.borrow() {
            STATE_DIRTY.store(true, Ordering::Relaxed);
        }
    })
}

pub(crate) fn clear_state_dirty() {
    STATE_DIRTY.store(false, Ordering::Relaxed);
}
