use std::any::Any;
use std::rc::Rc;
use std::cell::{Ref, RefCell};

pub(crate) struct StateHolder {
    pub state: Box<dyn Any>,
    pub dirty: bool,
}

pub(crate) struct StateHolder2 {
    pub state: Rc<dyn Any>,
    pub dirty: bool,
}

impl StateHolder2 {
    pub fn borrow<T: 'static>(&self) -> Ref<'_, T> {
        self
            .state
            .downcast_ref::<RefCell<T>>()
            .unwrap()
            .borrow()
    }
}