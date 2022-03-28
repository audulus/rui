use std::cell::RefCell;
use std::rc::Rc;

use crate::*;

struct Holder<S> {
    value: S,

    /// Has the state changed since the last redraw?
    dirty: Rc<RefCell<bool>>,
}

/// Contains application state. Application state is created using `state`.
#[derive(Clone)]
pub struct State<S> {
    value: Rc<RefCell<Holder<S>>>,
}

impl<S> State<S> {
    pub fn new(value: S, dirty: Rc<RefCell<bool>>) -> Self {
        Self {
            value: Rc::new(RefCell::new(Holder { value, dirty })),
        }
    }
}

impl<S> Binding<S> for State<S>
where
    S: Clone + 'static,
{
    fn with<T, F: Fn(&S) -> T>(&self, f: F) -> T {
        f(&self.value.borrow().value)
    }
    fn with_mut<T, F: Fn(&mut S) -> T>(&self, f: F) -> T {
        let mut holder = self.value.borrow_mut();
        // Set dirty so the view tree will be redrawn.
        *holder.dirty.borrow_mut() = true;
        f(&mut holder.value)
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
