use std::any::Any;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use tao::event_loop::EventLoopProxy;
use crate::*;

static STATE_DIRTY: AtomicBool = AtomicBool::new(false);

pub(crate) fn is_state_dirty() -> bool {
    STATE_DIRTY.load(Ordering::Relaxed)
}

pub(crate) fn set_state_dirty() {
    STATE_DIRTY.store(true, Ordering::Relaxed);
}

pub(crate) fn clear_state_dirty() {
    STATE_DIRTY.store(false, Ordering::Relaxed);
}

struct Holder<S> {
    value: S,
}

/// Contains application state. Application state is created using `state`.
#[derive(Clone)]
pub struct State<S> {
    value: Arc<Mutex<Holder<S>>>,
    event_loop_proxy: Option<EventLoopProxy<()>>,
}

impl<S> State<S> {
    pub fn new(value: S, event_loop_proxy: Option<EventLoopProxy<()>>) -> Self {
        Self {
            value: Arc::new(Mutex::new(Holder { value })),
            event_loop_proxy,
        }
    }
}

impl<S> AnyState for State<S>
where
    S: 'static,
{
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
        set_state_dirty();
        let t = f(&mut holder.value);

        // Wake up the event loop.
        if let Some(proxy) = &self.event_loop_proxy {
            if let Err(err) = proxy.send_event(()) {
                println!("error waking up event loop: {:?}", err);
            }
        }

        t
    }
}

impl<S: std::ops::Add + Copy + 'static> std::ops::Add<S> for &State<S> {
    type Output = <S as std::ops::Add>::Output;

    fn add(self, other: S) -> Self::Output {
        self.get() + other
    }
}

impl<T: std::ops::Add<Output = T> + Copy + 'static> std::ops::AddAssign<T> for State<T> {
    fn add_assign(&mut self, rhs: T) {
        self.set( self.get() + rhs )
    }
}

impl<T: std::ops::Sub<Output = T> + Copy + 'static> std::ops::SubAssign<T> for State<T> {
    fn sub_assign(&mut self, rhs: T) {
        self.set( self.get() - rhs )
    }
}

struct StateView<D, F> {
    default: D,
    func: F,
}

impl<S, V, D, F> View for StateView<D, F>
where
    V: View,
    S: Clone + 'static,
    D: Fn() -> S,
    F: Fn(State<S>) -> V,
{
    fn print(&self, id: ViewID, cx: &mut Context) {
        let s = cx.get_state(id, &self.default);
        (self.func)(s).print(id.child(&0), cx);
    }

    fn process(&self, event: &Event, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        let s = cx.get_state(id, &self.default);
        (self.func)(s).process(event, id.child(&0), cx, vger);
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        let s = cx.get_state(id, &self.default);
        (self.func)(s).draw(id.child(&0), cx, vger);
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        let s = cx.get_state(id, &self.default);
        (self.func)(s).layout(id.child(&0), sz, cx, vger)
    }

    fn hittest(
        &self,
        id: ViewID,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewID> {
        let s = cx.get_state(id, &self.default);
        (self.func)(s).hittest(id.child(&0), pt, cx, vger)
    }

    fn commands(&self, id: ViewID, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        let s = cx.get_state(id, &self.default);
        (self.func)(s).commands(id.child(&0), cx, cmds);
    }

    fn gc(&self, id: ViewID, cx: &mut Context, map: &mut StateMap) {
        let s = cx.get_state(id, &self.default);
        map.insert(id, Box::new(s.clone()));
        (self.func)(s).gc(id.child(&0), cx, map);
    }

    fn access(
        &self,
        id: ViewID,
        cx: &mut Context,
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        let s = cx.get_state(id, &self.default);
        (self.func)(s).access(id.child(&0), cx, nodes)
    }
}

impl<S, F> private::Sealed for StateView<S, F> {}

/// State allows you to associate some state with a view.
/// This is what you'll use for a data model, as well as per-view state.
/// Your state should be efficiently clonable. Use Rc as necessary.
///
/// `initial` is the initial value for your state.
///
/// `f` callback which is passed a `State<S>`
pub fn state<
    S: Clone + 'static,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state() {
        let _ = state(|| 0, |_s: State<usize>| EmptyView {});
    }

    #[test]
    fn test_state_clone() {
        let s = State::new(0, None);
        let s2 = s.clone();
        s.set(42);
        assert_eq!(s2.get(), 42);
    }
}

struct StateView2<D, F> {
    default: D,
    func: F,
}

impl<S, V, D, F> View for StateView2<D, F>
where
    V: View,
    S: Clone + 'static,
    D: Fn() -> S,
    F: Fn(&mut S) -> V,
{
    fn print(&self, id: ViewID, cx: &mut Context) {
        cx.get_state(id, &self.default).with_mut(|x|
            (self.func)(x).print(id.child(&0), cx)
        )
    }

    fn process(&self, event: &Event, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        cx.get_state(id, &self.default).with_mut(|x|
            (self.func)(x).process(event, id.child(&0), cx, vger)
        )
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        cx.get_state(id, &self.default).with_mut(|x|
            (self.func)(x).draw(id.child(&0), cx, vger)
        )
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        cx.get_state(id, &self.default).with_mut(|x|
            (self.func)(x).layout(id.child(&0), sz, cx, vger)
        )
    }

    fn hittest(
        &self,
        id: ViewID,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewID> {
        cx.get_state(id, &self.default).with_mut(|x|
            (self.func)(x).hittest(id.child(&0), pt, cx, vger)
        )
    }

    fn commands(&self, id: ViewID, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        cx.get_state(id, &self.default).with_mut(|x|
            (self.func)(x).commands(id.child(&0), cx, cmds)
        )
    }

    fn gc(&self, id: ViewID, cx: &mut Context, map: &mut StateMap) {
        // cx.with_state_aux(&self.default, id, map, |state: State<S>, cx, map| {
        //     map.insert(id, Box::new(state.clone()));
        //     (self.func)(state.clone()).gc(id.child(&0), cx, map);
        // });
    }

    fn access(
        &self,
        id: ViewID,
        cx: &mut Context,
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        cx.get_state(id, &self.default).with_mut(|x|
            (self.func)(x).access(id.child(&0), cx, nodes)
        )
    }
}

impl<S, F> private::Sealed for StateView2<S, F> {}

pub fn state2<
    S: Clone + 'static,
    V: View,
    D: Fn() -> S + 'static,
    F: Fn(&mut S) -> V + 'static,
>(
    initial: D,
    f: F,
) -> impl View + 'static {
    StateView2 {
        default: initial,
        func: f
    }
}