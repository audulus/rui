use std::any::{Any, TypeId};
use std::ops::{Index, IndexMut};
use std::rc::Rc;
use std::cell::{Cell, RefCell};

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
