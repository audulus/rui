use std::any::Any;
use std::sync::Mutex;
use tao::event_loop::EventLoopProxy;

use crate::*;

lazy_static! {
    /// Blasphemy!
    static ref GLOBAL_STATE_MAP: Mutex<HashMap<ViewID, Box<dyn Any + Send>>> = Mutex::new(HashMap::new());

    pub(crate) static ref GLOBAL_EVENT_LOOP_PROXY: Mutex<Option<EventLoopProxy<()>>> = Mutex::new(None);
}

/// Contains application state.
#[derive(Clone, Copy)]
pub struct GState<S> {
     id: ViewID,
     phantom: std::marker::PhantomData<S>,
}

impl<S> GState<S> 
where 
    S: Send + 'static
{
    pub fn new(id: ViewID, default: &impl Fn() -> S) -> Self {
        let mut map = GLOBAL_STATE_MAP.lock().unwrap();
        map.entry(id)
                   .or_insert_with(|| Box::new(default()));
        Self {
            id,
            phantom: Default::default()
        }
    }
}

impl<S> Binding<S> for GState<S>
where
    S: Clone + Send + Default + 'static,
{
    fn with<T, F: FnOnce(&S) -> T>(&self, f: F) -> T {
        let mut map = GLOBAL_STATE_MAP.lock().unwrap();
        let s = map.entry(self.id)
                   .or_insert_with(|| Box::new(S::default()));
        if let Some(state) = s.downcast_ref::<S>() {
            f(&state)
        } else {
            panic!("state has wrong type")
        }
    }
    fn with_mut<T, F: FnOnce(&mut S) -> T>(&self, f: F) -> T {
        let mut map = GLOBAL_STATE_MAP.lock().unwrap();
        let s = map.entry(self.id)
                       .or_insert_with(|| Box::new(S::default()));
        set_state_dirty();
        let t = if let Some(mut state) = s.downcast_mut::<S>() {
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

struct GStateView<D, F> {
    default: D,
    func: F,
}

impl<S, V, D, F> View for GStateView<D, F>
where
    V: View,
    S: Clone + Send + 'static,
    D: Fn() -> S,
    F: Fn(GState<S>) -> V,
{
    fn print(&self, id: ViewID, cx: &mut Context) {
        (self.func)(GState::new(id, &self.default)).print(id.child(&0), cx);
    }

    fn process(&self, event: &Event, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        (self.func)(GState::new(id, &self.default)).process(event, id.child(&0), cx, vger);
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        (self.func)(GState::new(id, &self.default)).draw(id.child(&0), cx, vger);
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        (self.func)(GState::new(id, &self.default)).layout(id.child(&0), sz, cx, vger)
    }

    fn hittest(
        &self,
        id: ViewID,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewID> {
        (self.func)(GState::new(id, &self.default)).hittest(id.child(&0), pt, cx, vger)
    }

    fn commands(&self, id: ViewID, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        (self.func)(GState::new(id, &self.default)).commands(id.child(&0), cx, cmds);
    }

    fn gc(&self, id: ViewID, cx: &mut Context, map: &mut StateMap) {
        (self.func)(GState::new(id, &self.default)).gc(id.child(&0), cx, map);
    }

    fn access(
        &self,
        id: ViewID,
        cx: &mut Context,
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        (self.func)(GState::new(id, &self.default)).access(id.child(&0), cx, nodes)
    }
}

impl<S, F> private::Sealed for GStateView<S, F> {}

/// State allows you to associate some state with a view.
/// This is what you'll use for a data model, as well as per-view state.
/// Your state should be efficiently clonable. Use Rc as necessary.
///
/// `initial` is the initial value for your state.
///
/// `f` callback which is passed a `State<S>`
pub fn gstate<
    S: Clone + Send + 'static,
    V: View + 'static,
    D: Fn() -> S + 'static,
    F: Fn(GState<S>) -> V + 'static,
>(
    initial: D,
    f: F,
) -> impl View + 'static {
    GStateView {
        default: initial,
        func: f,
    }
}
