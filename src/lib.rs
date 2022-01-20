use std::any::{Any, TypeId};
use std::ops::{Index, IndexMut};
use std::rc::Rc;
use std::cell::{Cell, RefCell, RefMut};

pub trait View { }

pub struct EmptyView { }

impl View for EmptyView { }

pub struct State<'a, S, V: View> { 
    func: Box<dyn Fn(Rc<S>) -> V + 'a>
}

impl<'a, S, V> View for State<'a, S, V> where V: View { }

pub fn state<'a, S, V: View, F: Fn(Rc<S>) -> V + 'a>(f: F) -> State<'a, S, V> {
    State{func: Box::new(f)}
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

impl View2 for Counter {
    fn body(&self) -> Box<dyn View2> {
        let count = self.count.clone();
        Box::new(Button2::new(format!("{:?}", count.get()).as_str(), move || {
            count.set(*count.get() + 1);
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
        let _ = state(|_s: Rc<usize>| {
            EmptyView{}
        });
    }

    fn counter() -> Box<dyn View> {
        Box::new(state(|count: Rc<RefCell<usize>>| {
            button(format!("{:?}", (*count)).as_str(), move || {
                *count.borrow_mut() += 1;
            })
        }))
    }

    #[test]
    fn test_state2() {
        let _ = counter();
    }
}
