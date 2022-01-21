// #![feature(type_alias_impl_trait)]

use std::cell::{RefCell, RefMut};
use std::rc::Rc;

pub trait Binding<S> {
    fn get(&self) -> RefMut<'_, S>;
}

#[derive(Clone)]
pub struct State<S> {
    value: Rc<RefCell<S>>,
}

impl<S> State<S> {
    fn new(value: S) -> Self {
        Self {
            value: Rc::new(RefCell::new(value)),
        }
    }

    fn set(&self, value: S) {
        *self.value.borrow_mut() = value;
    }
}

impl<S> Binding<S> for State<S> {
    fn get(&self) -> RefMut<'_, S> {
        // Here we can indicate that a state change has
        // been made.
        self.value.borrow_mut()
    }
}

pub trait View {
    fn draw(&self);
}

pub struct EmptyView {}

impl View for EmptyView {
    fn draw(&self) {
        println!("EmptyView");
    }
}

pub struct StateView<S, V: View> {
    initial: S,
    func: Box<dyn Fn(State<S>) -> V>,
}

impl<S, V> View for StateView<S, V> where V: View, S: Clone {
    fn draw(&self) {
        // XXX: of course we need to actually pull the state
        //      from a context.
        let s = State::new(self.initial.clone());
        (*self.func)(s).draw();
    }
}

pub fn state<S: Clone, V: View, F: Fn(State<S>) -> V + 'static>(initial: S, f: F) -> StateView<S, V> {
    StateView { initial: initial, func: Box::new(f) }
}

pub struct Button {
    text: String,
    func: Box<dyn Fn()>,
}

impl View for Button {
    fn draw(&self) {
        println!("Button({:?})", self.text);
    }
}

pub fn button<F: Fn() + 'static>(name: &str, f: F) -> Button {
    Button {
        text: String::from(name),
        func: Box::new(f),
    }
}

pub struct Stack {
    children: Vec<Box<dyn View>>,
}

impl View for Stack {

    fn draw(&self) {
        println!("Stack {{");
        for child in &self.children {
            (*child).draw();
        }
        println!("}}");
    }

}

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
        let _ = state(0, |_s: State<usize>| EmptyView {});
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
        let v = counter(42);
        v.draw();
    }

    #[test]
    fn test_stack() {
        let mut s = Stack::new();
        s.push(EmptyView {});
        s.push(button("click me!", || {
            println!("clicked");
        }));
        s.draw();
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
        let v = counter2(42);
        v.draw();
    }

    fn counter3<B>(count: B) -> impl View where B : Binding<usize> + Clone + 'static {
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

    #[test]
    fn test_binding() {
        let _ = state(42, |count: State<usize>| {
            counter3(count)
        });
    }

    fn ok_button<F: Fn() + 'static>(f: F) -> impl View {
        button("ok", f)
    }
}
