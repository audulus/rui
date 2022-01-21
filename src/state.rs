use std::cell::{RefCell, RefMut};
use std::rc::Rc;

pub trait Binding<S> {
    fn get(&self) -> RefMut<'_, S>;
}

pub trait AnyState { }

#[derive(Clone)]
pub struct State<S> {
    value: Rc<RefCell<S>>,
}

impl<S> State<S> {
    pub fn new(value: S) -> Self {
        Self {
            value: Rc::new(RefCell::new(value)),
        }
    }

    pub fn set(&self, value: S) {
        *self.value.borrow_mut() = value;
    }
}

impl<S> AnyState for State<S> { }

impl<S> Binding<S> for State<S> {
    fn get(&self) -> RefMut<'_, S> {
        // Here we can indicate that a state change has
        // been made.
        self.value.borrow_mut()
    }
}

