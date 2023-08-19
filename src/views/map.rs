use crate::*;
use std::any::Any;

pub struct MapView<S1, SF, F> {
    value: S1,
    set_value: SF,
    func: F,
}

impl<S1, V, SF, F> View for MapView<S1, SF, F>
where
    V: View,
    S1: Clone + 'static,
    SF: Fn(S1, &mut Context) + 'static,
    F: Fn(StateHandle<S1>, &mut Context) -> V + 'static,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let id = hash(path);
        cx.set_state(id, self.value.clone());
        let s = StateHandle::new(id);
        path.push(0);
        (self.func)(s, cx).process(event, path, cx, actions);
        path.pop();

        // If processing the event changed the state, then call the set_value function.
        if cx.is_dirty(id) {
            (self.set_value)(cx[s].clone(), cx)
        }
    }

    fn draw(&self, path: &mut IdPath, args: &mut DrawArgs) {
        let id = hash(path);
        args.cx.set_state(id, self.value.clone());
        path.push(0);
        (self.func)(StateHandle::new(id), args.cx).draw(path, args);
        path.pop();
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        let id = hash(path);
        args.cx.set_state(id, self.value.clone());

        path.push(0);
        let sz = (self.func)(StateHandle::new(id), args.cx).layout(path, args);
        path.pop();
        sz
    }

    fn dirty(&self, path: &mut IdPath, xform: LocalToWorld, cx: &mut Context) {
        let id = hash(path);
        cx.set_state(id, self.value.clone());
        path.push(0);
        (self.func)(StateHandle::new(id), cx).dirty(path, xform, cx);
        path.pop();
    }

    fn hittest(&self, path: &mut IdPath, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let id = hash(path);
        cx.set_state(id, self.value.clone());
        path.push(0);
        let hit_id = (self.func)(StateHandle::new(id), cx).hittest(path, pt, cx);
        path.pop();
        hit_id
    }

    fn commands(&self, path: &mut IdPath, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        let id = hash(path);
        cx.set_state(id, self.value.clone());
        path.push(0);
        (self.func)(StateHandle::new(id), cx).commands(path, cx, cmds);
        path.pop();
    }

    fn gc(&self, path: &mut IdPath, cx: &mut Context, map: &mut Vec<ViewId>) {
        let id = hash(path);
        cx.set_state(id, self.value.clone());
        map.push(id);
        path.push(0);
        (self.func)(StateHandle::new(id), cx).gc(path, cx, map);
        path.pop();
    }

    fn access(
        &self,
        path: &mut IdPath,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        let id = hash(path);
        cx.set_state(id, self.value.clone());
        path.push(0);
        let node_id = (self.func)(StateHandle::new(id), cx).access(path, cx, nodes);
        path.pop();
        node_id
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
