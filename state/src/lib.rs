use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::{Ref, RefMut, RefCell};

type StateMap = HashMap<u64, StateHolder>;

struct Runtime {
    state_map: RefCell<StateMap>
}

impl Runtime {
    fn new() -> Self {
        Runtime { state_map: RefCell::new(HashMap::new()) }
    }
}

thread_local! {
    static RUNTIME: Runtime = Runtime::new();
}

struct StateHolder {
    value: Rc<dyn Any>,
    dirty: bool,
}

impl StateHolder {

    pub fn new<T: 'static>(value: T) -> Self {
        StateHolder { value: Rc::new(RefCell::new(value)), dirty: false }
    }

    pub fn borrow<T: 'static>(&self) -> Ref<'_, T> {
        self
            .value
            .downcast_ref::<RefCell<T>>()
            .unwrap()
            .borrow()
    }

    pub fn borrow_mut<T: 'static>(&self) -> RefMut<'_, T> {
        self
            .value
            .downcast_ref::<RefCell<T>>()
            .unwrap()
            .borrow_mut()
    }
}

#[derive(Clone, Copy)]
struct StateHandle<T> {
    id: u64,
    phantom: std::marker::PhantomData<T>,
}

impl<T: 'static> StateHandle<T> {
    pub fn new(id: u64, value: T) -> Self {
        RUNTIME.with(move |runtime| {
            runtime.state_map.borrow_mut().insert(id, StateHolder::new(value))
        });
        Self {
            id,
            phantom: Default::default(),
        }
    }

    pub fn value(&self) -> Rc<dyn Any> {
        let id = self.id;
        RUNTIME.with(|runtime| {
            runtime.state_map.borrow().get(&id).unwrap().value.clone()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_handle() {
        let handle = StateHandle::new(0, 42 as i64);

        let rc = handle.value();

        assert_eq!(*rc.downcast_ref::<RefCell<i64>>().unwrap().borrow(), 42);

    }
}
