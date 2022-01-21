// #![feature(type_alias_impl_trait)]

mod view;
pub use view::*;

mod state;
pub use state::*;

mod text;
pub use text::*;

mod button;
pub use button::*;

mod stack;
pub use stack::*;

pub struct StateView<S, V: View> {
    state: State<S>,
    func: Box<dyn Fn(State<S>) -> V>,
}

impl<S, V> View for StateView<S, V> where V: View, S: Clone {
    fn draw(&self) {
        (*self.func)(self.state.clone()).draw();
    }
    fn process(&self, event: &Event) {
        (*self.func)(self.state.clone()).process(event);
    }
}

pub fn state<S: Clone, V: View, F: Fn(State<S>) -> V + 'static>(initial: S, f: F) -> StateView<S, V> {
    StateView { state: State::new(initial), func: Box::new(f) }
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

    fn counter0(start: usize) -> impl View {
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
        let s = stack2(
            EmptyView{},
            button("click me!", || {
                println!("clicked");
            })
        );
        s.draw();
    }

    fn counter(start: usize) -> impl View {
        state(start, |count: State<usize>| {
            let count2 = count.clone();
            let value_string = format!("value: {:?}", *count.get());
            stack3(
                text(value_string.as_str()),
                button("increment", move || {
                    *count.get() += 1;
                }),
                button("decrement", move || {
                    *count2.get() -= 1;
                })
            )
        })
    }

    #[test]
    fn test_state3() {
        let v = counter(42);
        println!("\"drawing\" the UI");
        v.draw();
        println!("ok, now pressing increment button");
        v.process(&Event::PressButton(String::from("increment")));
        println!("\"drawing\" the UI again");
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
