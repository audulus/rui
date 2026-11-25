use crate::*;
use std::any::Any;

#[derive(Clone)]
pub struct MapView<S1, SF, F> {
    value: S1,
    set_value: SF,
    func: F,
}

impl<S1, V, SF, F> DynView for MapView<S1, SF, F>
where
    V: View,
    S1: Clone + 'static,
    SF: Fn(S1, &mut Context) + Clone + 'static,
    F: Fn(StateHandle<S1>, &mut Context) -> V + Clone + 'static,
{
    fn process(
        &self,
        event: &Event,
        vid: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let id = vid;
        cx.set_state(id, self.value.clone());
        let s = StateHandle::new(id);
        let cid = cx.child(vid, 0);
        (self.func)(s, cx).process(event, cid, cx, actions);

        // If processing the event changed the state, then call the set_value function.
        if cx.is_dirty(id) {
            (self.set_value)(cx[s].clone(), cx)
        }
    }

    fn draw(&self, vid: ViewId, args: &mut DrawArgs) {
        let id = vid;
        args.cx.set_state(id, self.value.clone());
        let cid = args.cx.child(vid, 0);
        (self.func)(StateHandle::new(id), args.cx).draw(cid, args);
    }

    fn layout(&self, vid: ViewId, args: &mut LayoutArgs) -> LocalSize {
        let id = vid;
        args.cx.set_state(id, self.value.clone());

        let cid = args.cx.child(vid, 0);

        (self.func)(StateHandle::new(id), args.cx).layout(cid, args)
    }

    fn dirty(&self, vid: ViewId, xform: LocalToWorld, cx: &mut Context) {
        let id = vid;
        cx.set_state(id, self.value.clone());
        let cid = cx.child(vid, 0);
        (self.func)(StateHandle::new(id), cx).dirty(cid, xform, cx);
    }

    fn hittest(&self, vid: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let id = vid;
        cx.set_state(id, self.value.clone());
        let cid = cx.child(vid, 0);

        (self.func)(StateHandle::new(id), cx).hittest(cid, pt, cx)
    }

    fn commands(&self, vid: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        let id = vid;
        cx.set_state(id, self.value.clone());
        let cid = cx.child(vid, 0);
        (self.func)(StateHandle::new(id), cx).commands(cid, cx, cmds);
    }

    fn access(
        &self,
        vid: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        let id = vid;
        cx.set_state(id, self.value.clone());
        let cid = cx.child(vid, 0);

        (self.func)(StateHandle::new(id), cx).access(cid, cx, nodes)
    }
}

impl<S1, SF, F> private::Sealed for MapView<S1, SF, F> {}

/// Maps state into local state.
///
/// For example:
///
/// ```no_run
/// # use rui::*;
///
/// #[derive(Debug, Default)]
/// struct MyState {
///     x: f32,
/// }
///
/// fn main() {
///     rui(state(MyState::default, |state, cx| {
///         vstack((
///             format!("value: {:?}", cx[state]).padding(Auto),
///             map(
///                 cx[state].x * 0.01,
///                 move |v, cx| cx[state].x = v * 100.0,
///                 |s, _| knob(s).padding(Auto),
///             ),
///         ))
///     }));
/// }
/// ```
pub fn map<S, SF, F>(value: S, set_value: SF, func: F) -> impl View
where
    MapView<S, SF, F>: view::View,
{
    MapView {
        value,
        set_value,
        func,
    }
}
