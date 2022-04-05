use std::sync::{Arc, Mutex};
use std::any::Any;

use crate::*;

struct Holder<S> {
    value: S,

    /// Has the state changed since the last redraw?
    dirty: Arc<Mutex<Dirty>>,
}

/// Contains application state. Application state is created using `state`.
#[derive(Clone)]
pub struct State<S> {
    value: Arc<Mutex<Holder<S>>>,
}

impl<S> State<S> {
    pub fn new(value: S, dirty: Arc<Mutex<Dirty>>) -> Self {
        Self {
            value: Arc::new(Mutex::new(Holder { value, dirty })),
        }
    }
}

impl<S> AnyState for State<S> where S: 'static {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<S> Binding<S> for State<S>
where
    S: Clone + 'static,
{
    fn with<T, F: FnOnce(&S) -> T>(&self, f: F) -> T {
        f(&self.value.lock().unwrap().value)
    }
    fn with_mut<T, F: FnOnce(&mut S) -> T>(&self, f: F) -> T {
        let mut holder = self.value.lock().unwrap();
        // Set dirty so the view tree will be redrawn.
        holder.dirty.lock().unwrap().dirty = true;
        let t = f(&mut holder.value);

        // Wake up the event loop.
        if let Some(proxy) = &holder.dirty.lock().unwrap().event_loop_proxy {
            if let Err(err) = proxy.send_event( () ) {
                println!("error waking up event loop: {:?}", err);
            }
        }

        t
    }
}

struct StateView<S: 'static, V: View, F: Fn(State<S>) -> V> {
    default: S,
    func: F,
}

impl<S, V, F> View for StateView<S, V, F>
where
    V: View,
    S: Clone,
    F: Fn(State<S>) -> V,
{
    fn print(&self, id: ViewID, cx: &mut Context) {
        cx.with_state(self.default.clone(), id, |state: State<S>, cx| {
            (self.func)(state.clone()).print(id.child(&0), cx);
        });
    }

    fn process(&self, event: &Event, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        cx.with_state_vger(
            vger,
            self.default.clone(),
            id,
            |state: State<S>, cx, vger| {
                (self.func)(state.clone()).process(event, id.child(&0), cx, vger);
            },
        )
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        cx.with_state_vger(
            vger,
            self.default.clone(),
            id,
            |state: State<S>, cx, vger| {
                (self.func)(state.clone()).draw(id.child(&0), cx, vger);
            },
        );
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        cx.with_state_vger(
            vger,
            self.default.clone(),
            id,
            |state: State<S>, cx, vger| {
                (self.func)(state.clone()).layout(id.child(&0), sz, cx, vger)
            },
        )
    }

    fn hittest(
        &self,
        id: ViewID,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewID> {
        cx.with_state_vger(
            vger,
            self.default.clone(),
            id,
            |state: State<S>, cx, vger| {
                (self.func)(state.clone()).hittest(id.child(&0), pt, cx, vger)
            },
        )
    }

    fn commands(&self, id: ViewID, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        cx.with_state_mut(self.default.clone(), id, &mut |state: State<S>, cx| {
            (self.func)(state.clone()).commands(id.child(&0), cx, cmds);
        });
    }

    fn gc(&self, id: ViewID, cx: &mut Context, map: &mut StateMap) {
        cx.with_state_gc(map, self.default.clone(), id, |state: State<S>, cx, map| {
            map.insert(id, Box::new(state.clone()));
            (self.func)(state.clone()).gc(id.child(&0), cx, map);
        });
    }

    fn access(&self, id: ViewID, cx: &mut Context, nodes: &mut Vec<accesskit::Node>) -> Option<accesskit::NodeId> {
        cx.with_state_access(nodes, self.default.clone(), id, |state: State<S>, cx, nodes| {
            (self.func)(state.clone()).access(id.child(&0), cx, nodes)
        })
    }
}

/// State allows you to associate some state with a view.
/// This is what you'll use for a data model, as well as per-view state.
/// Your state should be efficiently clonable. Use Rc as necessary.
///
/// `initial` is the initial value for your state.
///
/// `f` callback which is passed a `State<S>`
pub fn state<S: Clone + 'static, V: View + 'static, F: Fn(State<S>) -> V + 'static>(
    initial: S,
    f: F,
) -> impl View + 'static {
    StateView {
        default: initial,
        func: f,
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn test_state() {
        let _ = state(0, |_s: State<usize>| EmptyView {});
    }

    #[test]
    fn test_state_clone() {
        let d = Arc::new(Mutex::new(Dirty::new(None)));
        let s = State::new(0, d);
        let s2 = s.clone();
        s.set(42);
        assert_eq!(s2.get(), 42);
    }

}