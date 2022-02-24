use std::cell::{RefCell, RefMut};
use std::rc::Rc;

use crate::*;

pub trait Binding<S> {
    fn get(&self) -> S;
    fn set(&self, value: S);
}

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
}

impl<S> Binding<S> for State<S> where S: Clone {
    fn get(&self) -> S {
        // Here we can indicate that a state change has
        // been made.
        self.value.borrow().clone()
    }
    fn set(&self, value: S) {
        *self.value.borrow_mut() = value.clone();
    }
}

pub struct StateView<S: 'static, V: View> {
    default: S,
    func: Box<dyn Fn(State<S>) -> V>,
}

impl<S, V> View for StateView<S, V>
where
    V: View,
    S: Clone,
{
    fn print(&self, id: ViewID, cx: &mut Context) {
        cx.with_state(self.default.clone(), id, |state: State<S>, cx| {
            (*self.func)(state.clone()).print(id.child(0), cx);
        });
    }

    fn process(&self, event: &Event, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        cx.with_state_vger(vger, self.default.clone(), id, |state: State<S>, cx, vger| {
            (*self.func)(state.clone()).process(event, id.child(0), cx, vger);
        })
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        cx.with_state_vger(
            vger,
            self.default.clone(),
            id,
            |state: State<S>, cx, vger| {
                (*self.func)(state.clone()).draw(id.child(0), cx, vger);
            },
        );
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        cx.with_state_vger(
            vger,
            self.default.clone(),
            id,
            |state: State<S>, cx, vger| {
                (*self.func)(state.clone()).layout(id.child(0), sz, cx, vger)
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
                (*self.func)(state.clone()).hittest(id.child(0), pt, cx, vger)
            },
        )
    }
}

pub fn state<S: Clone, V: View, F: Fn(State<S>) -> V + 'static>(
    initial: S,
    f: F,
) -> StateView<S, V> {
    StateView {
        default: initial,
        func: Box::new(f),
    }
}

pub struct ValueBinding<S> {
    pub getf: Box<dyn Fn() -> S >,
    pub setf: Box<dyn Fn(S)>,
}

impl<S> Binding<S> for ValueBinding<S> {
    fn get(&self) -> S {
        (*self.getf)()
    }
    fn set(&self, value: S) {
        (*self.setf)(value);
    }
}

#[macro_export]
macro_rules! bind {
    ( $state:expr, $field:ident ) => {
        {
            let state1 = $state.clone();
            let state2 = $state.clone();
            ValueBinding {
                getf: Box::new(move || { state1.get().$field.clone() }),
                setf: Box::new(move |val| { 
                    let mut s = state2.get();
                    s.$field = val;
                    state2.set(s);
                } )
            }
        }
    };
}