use std::any::Any;
use std::sync::{Arc, Mutex};
use tao::event_loop::EventLoopProxy;
use std::sync::atomic::{AtomicBool, Ordering};
use std::cell::RefCell;

use crate::*;

static STATE_DIRTY: AtomicBool = AtomicBool::new(false);

thread_local! {
    pub static ENABLE_DIRTY: RefCell<bool> = RefCell::new(true);
}

pub(crate) fn is_state_dirty() -> bool {
    STATE_DIRTY.load(Ordering::Relaxed)
}

pub(crate) fn set_state_dirty() {
    ENABLE_DIRTY.with(|enable| {
        if *enable.borrow() {
            STATE_DIRTY.store(true, Ordering::Relaxed);
        }
    })
}

pub(crate) fn clear_state_dirty() {
    STATE_DIRTY.store(false, Ordering::Relaxed);
}

pub(crate) type StateMap = HashMap<ViewID, Arc<Mutex<dyn Any + Send>>>;

lazy_static! {
    /// Global map for storing state values.
    pub(crate) static ref GLOBAL_STATE_MAP: Mutex<StateMap> = Mutex::new(StateMap::new());

    /// Allows us to wake the event loop whenever we want.
    pub(crate) static ref GLOBAL_EVENT_LOOP_PROXY: Mutex<Option<EventLoopProxy<()>>> = Mutex::new(None);
}

/// Weak reference to app state.
#[derive(Clone)]
pub struct State<S> {
     id: ViewID,
     phantom: std::marker::PhantomData<S>,
}

impl<S> Copy for State<S> where S: Clone { }

impl<S> State<S> 
where 
    S: Send + 'static
{
    pub fn new(id: ViewID, default: &impl Fn() -> S) -> Self {
        let mut map = GLOBAL_STATE_MAP.lock().unwrap();
        map.entry(id)
                   .or_insert_with(|| Arc::new(Mutex::new(default())));
        Self {
            id,
            phantom: Default::default()
        }
    }
}

impl<S> Binding<S> for State<S>
where
    S: Clone + Send + 'static,
{
    fn with<T, F: FnOnce(&S) -> T>(&self, f: F) -> T {
        let s = {
            let map = GLOBAL_STATE_MAP.lock().unwrap();
            map[&self.id].clone()
        };
        let v = s.lock().unwrap();
        if let Some(state) = v.downcast_ref::<S>() {
            f(&state)
        } else {
            panic!("state has wrong type")
        }
    }
    fn with_mut<T, F: FnOnce(&mut S) -> T>(&self, f: F) -> T {
        let s = {
            let map = GLOBAL_STATE_MAP.lock().unwrap();
            map[&self.id].clone()
        };
        set_state_dirty();
        let t = if let Some(mut state) = s.lock().unwrap().downcast_mut::<S>() {
            f(&mut state)
        } else {
            panic!("state has wrong type")
        };

        // Wake up the event loop.
        let opt_proxy = GLOBAL_EVENT_LOOP_PROXY.lock().unwrap();
        if let Some(proxy) = &*opt_proxy {
            if let Err(err) = proxy.send_event(()) {
                println!("error waking up event loop: {:?}", err);
            }
        }

        t
    }
}

struct StateView<D, F> {
    default: D,
    func: F,
}

impl<S, V, D, F> View for StateView<D, F>
where
    V: View,
    S: Clone + Send + 'static,
    D: Fn() -> S,
    F: Fn(State<S>) -> V,
{
    fn print(&self, id: ViewID, cx: &mut Context) {
        (self.func)(State::new(id, &self.default)).print(id.child(&0), cx);
    }

    fn process(&self, event: &Event, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        (self.func)(State::new(id, &self.default)).process(event, id.child(&0), cx, vger);
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        (self.func)(State::new(id, &self.default)).draw(id.child(&0), cx, vger);
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        (self.func)(State::new(id, &self.default)).layout(id.child(&0), sz, cx, vger)
    }

    fn hittest(
        &self,
        id: ViewID,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewID> {
        (self.func)(State::new(id, &self.default)).hittest(id.child(&0), pt, cx, vger)
    }

    fn commands(&self, id: ViewID, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        (self.func)(State::new(id, &self.default)).commands(id.child(&0), cx, cmds);
    }

    fn gc(&self, id: ViewID, cx: &mut Context, map: &mut Vec<ViewID>) {
        map.push(id);
        (self.func)(State::new(id, &self.default)).gc(id.child(&0), cx, map);
    }

    fn access(
        &self,
        id: ViewID,
        cx: &mut Context,
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        (self.func)(State::new(id, &self.default)).access(id.child(&0), cx, nodes)
    }
}

impl<S, F> private::Sealed for StateView<S, F> {}

/// State allows you to associate some state with a view.
/// This is what you'll use for a data model, as well as per-view state.
/// Your state should be efficiently clonable. Use Arc as necessary.
///
/// `initial` is the initial value for your state.
///
/// `f` callback which is passed a `State<S>`
pub fn state<
    S: Clone + Send + 'static,
    V: View + 'static,
    D: Fn() -> S + 'static,
    F: Fn(State<S>) -> V + 'static,
>(
    initial: D,
    f: F,
) -> impl View + 'static {
    StateView {
        default: initial,
        func: f,
    }
}
