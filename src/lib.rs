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

pub struct StateView<S, V: View> { 
    func: Box<dyn Fn(State<S>) -> V + 'static>
}

impl<S, V> View for StateView<S, V> where V: View { }

pub fn state<'a, S, V: View, F: Fn(State<S>) -> V + 'static>(_initial: S, f: F) -> StateView<S, V> {
    StateView{func: Box::new(f)}
}

pub struct Button {
    text: String,
    func: Box<dyn Fn() + 'static>
}

impl View for Button{ }

pub fn button<F: Fn() + 'static>(name: &str, f: F) -> Button {
    Button{text: String::from(name), func: Box::new(f)}
}

pub struct Stack {
    children: Vec<Box<dyn View>>
}

impl View for Stack { }

impl Stack {

    fn new() -> Self {
        Self { children: vec![] }
    }

    fn push(&mut self, view: impl View + 'static) {
        self.children.push(Box::new(view))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_state_clone() {
        let s = State::new(0);
        let s2 = s.clone();
        s.set(42);
        assert_eq!(*s2.get(), 42);
    }

    #[test]
    fn test_button() {
        let _ = button("click me", || {
            println!("clicked!");
        });
    }

    
    #[test]
    fn test_state() {
        let _ = state(0, |_s: State<usize>| {
            EmptyView{}
        });
    }

    fn counter(start: usize) -> impl View {
        state(start, |count: State<usize>| {
            button(format!("{:?}", *count.get()).as_str(), move || {
                *count.get() += 1;
            })
        })
    }

    #[test]
    fn test_state2() {
        let _ = counter(42);
    }

    #[test]
    fn test_stack() {
        let mut s = Stack::new();
        s.push(EmptyView{});
        s.push(button("click me!", || {
            println!("clicked");
        }))
    }

    fn counter2(start: usize) -> impl View {
        state(start, |count: State<usize>| {
            let count2 = count.clone();
            let mut stack = Stack::new();
            stack.push(button("increment", move || {
                *count.get() += 1;
            }));
            stack.push(button("decrement", move || {
                *count2.get() -= 1;
            }));
            stack
        })
    }

    #[test]
    fn test_state3() {
        let _ = counter2(42);
    }

    fn counter3(count: State<usize>) -> impl View {
        let count2 = count.clone();
        let mut stack = Stack::new();
        stack.push(button("increment", move || {
            *count.get() += 1;
        }));
        stack.push(button("decrement", move || {
            *count2.get() -= 1;
        }));
        stack
    }

    fn ok_button<F: Fn() + 'static>(f: F) -> impl View {
        button("ok", f)
    }
}
