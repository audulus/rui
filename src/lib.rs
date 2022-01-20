// #![feature(type_alias_impl_trait)]

use std::any::{Any, TypeId};
use std::ops::{Index, IndexMut};
use std::rc::Rc;
use std::cell::{Cell, RefCell, RefMut};

#[derive(Clone)]
pub struct State<S> {
    value: Rc<RefCell<S>>
}

impl<S> State<S> {
    
    fn new(value: S) -> Self {
        Self {
            value: Rc::new(RefCell::new(value))
        }
    }

    fn get(&self) -> RefMut<'_, S> {
        // Here we can indicate that a state change has
        // been made.
        self.value.borrow_mut()
    }

    fn set(&self, value: S) {
        *self.value.borrow_mut() = value;
    }
}

pub trait View { }

pub struct EmptyView { }

impl View for EmptyView { }

pub struct StateView<'a, S, V: View> { 
    func: Box<dyn Fn(State<S>) -> V + 'a>
}

impl<'a, S, V> View for StateView<'a, S, V> where V: View { }

pub fn state<'a, S, V: View, F: Fn(State<S>) -> V + 'a>(f: F) -> StateView<'a, S, V> {
    StateView{func: Box::new(f)}
}

pub struct Button<'a> {
    text: String,
    func: Box<dyn Fn() + 'a>
}

impl<'a> View for Button<'a> { }

pub fn button<'a, F: Fn() + 'a>(name: &str, f: F) -> Button<'a> {
    Button{text: String::from(name), func: Box::new(f)}
}

// More SwiftUI like

pub trait View2 {
    fn body(&self) -> Box<dyn View2>;
}

pub struct Bottom { }

impl View2 for Bottom { 
    fn body(&self) -> Box<dyn View2> {
        Box::new(Bottom{})
    }
}

pub struct Button2<'a> {
    text: String,
    func: Box<dyn Fn() + 'a>
}

impl<'a> Button2<'a> {
    
    fn new<F: Fn() + 'a>(name: &str, f: F) -> Self {
        Self {
            text: String::from(name),
            func: Box::new(f)
        }
    }
}

impl<'a> View2 for Button2<'a> {
    fn body(&self) -> Box<dyn View2> {
        Box::new(Bottom{})
    }
}

#[derive(Clone)]
pub struct State2<S> {
    value: Rc<RefCell<S>>
}

impl<S> State2<S> {
    
    fn new(value: S) -> Self {
        Self {
            value: Rc::new(RefCell::new(value))
        }
    }

    fn get(&self) -> RefMut<'_, S> {
        self.value.borrow_mut()
    }

    fn set(&self, value: S) {
        *self.value.borrow_mut() = value;
    }
}

pub struct Counter {
    count: State2<usize>
}

impl Counter {
    fn new() -> Self {
        Self {
            count: State2::new(0)
        }
    }
}

impl View2 for Counter {
    fn body(&self) -> Box<dyn View2> {
        let count = self.count.clone();
        Box::new(Button2::new(format!("{:?}", count.get()).as_str(), move || {
            *count.get() += 1;
        }))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_button() {
        let _ = button("click me", || {
            println!("clicked!");
        });
    }

    
    #[test]
    fn test_state() {
        let _ = state(|_s: State<usize>| {
            EmptyView{}
        });
    }

    fn counter() -> impl View {
        state(|count: State<usize>| {
            button(format!("{:?}", count.get()).as_str(), move || {
                *count.get() += 1;
            })
        })
    }

    #[test]
    fn test_state2() {
        let _ = counter();
    }
}
