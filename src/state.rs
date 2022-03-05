use std::cell::RefCell;
use std::rc::Rc;

use crate::*;

/// Reads or writes a value owned by a source-of-truth.
pub trait Binding<S>: Clone + 'static {
    fn get(&self) -> S;
    fn set(&self, value: S);
}

struct Holder<S> {
    value: S,

    /// Has the state changed since the last redraw?
    dirty: bool,
}

#[derive(Clone)]
pub struct State<S> {
    value: Rc<RefCell<Holder<S>>>,
}

impl<S> State<S> {
    pub fn new(value: S) -> Self {
        Self {
            value: Rc::new(RefCell::new(Holder {
                value,
                dirty: false,
            })),
        }
    }
    pub fn dirty(&self) -> bool {
        self.value.borrow().dirty
    }
    pub fn clear_dirty(&self) {
        self.value.borrow_mut().dirty = false
    }
}

impl<S> Binding<S> for State<S>
where
    S: Clone + 'static,
{
    fn get(&self) -> S {
        self.value.borrow().value.clone()
    }
    fn set(&self, value: S) {
        let mut holder = self.value.borrow_mut();
        holder.value = value.clone();

        // Set dirty so the view tree will be redrawn.
        holder.dirty = true;
    }
}

pub struct StateView<S: 'static, V: View, F: Fn(State<S>) -> V> {
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

    fn needs_redraw(&self, id: ViewID, cx: &mut Context) -> bool {
        cx.with_state(self.default.clone(), id, |state: State<S>, cx| {
            if state.dirty() {
                state.clear_dirty();
                true
            } else {
                (self.func)(state.clone()).needs_redraw(id.child(&0), cx)
            }
        })
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
}

/// State allows you to associate some state with a view.
/// This is what you'll use for a data model, as well as per-view state.
/// Your state should be efficiently clonable. Use Rc as necessary.
///
/// `initial` is the initial value for your state.
///
/// `f` callback which is passed a `State<S>`
pub fn state<S: Clone, V: View, F: Fn(State<S>) -> V + 'static>(
    initial: S,
    f: F,
) -> StateView<S, V, F> {
    StateView {
        default: initial,
        func: f,
    }
}

#[derive(Clone)]
pub struct Field<Get, Set> {
    pub getf: Get,
    pub setf: Set,
}

impl<S, Get, Set> Binding<S> for Field<Get, Set>
where
    Get: Fn() -> S + Clone + 'static,
    Set: Fn(S) + Clone + 'static,
{
    fn get(&self) -> S {
        (self.getf)()
    }
    fn set(&self, value: S) {
        (self.setf)(value);
    }
}

#[macro_export]
macro_rules! bind {
    ( $state:expr, $field:ident ) => {{
        let state1 = $state.clone();
        let state2 = $state.clone();
        Field {
            getf: move || state1.get().$field.clone(),
            setf: move |val| {
                let mut s = state2.get();
                s.$field = val;
                state2.set(s);
            },
        }
    }};
}
