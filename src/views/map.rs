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
    F: Fn(State<S1>, &mut Context) -> V + 'static,
{
    fn process(
        &self,
        event: &Event,
        id: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        cx.set_state(id, self.value.clone());
        let s = State::new(id);
        (self.func)(s, cx).process(event, id.child(&0), cx, actions);

        // If processing the event changed the state, then call the set_value function.
        if cx.is_dirty(id) {
            (self.set_value)(cx[s].clone(), cx)
        }
    }

    fn draw(&self, id: ViewId, args: &mut DrawArgs) {
        args.cx.set_state(id, self.value.clone());
        (self.func)(State::new(id), args.cx).draw(id.child(&0), args);
    }

    fn layout(&self, id: ViewId, args: &mut LayoutArgs) -> LocalSize {
        args.cx.set_state(id, self.value.clone());

        (self.func)(State::new(id), args.cx).layout(id.child(&0), args)
    }

    fn dirty(&self, id: ViewId, xform: LocalToWorld, cx: &mut Context) {
        cx.set_state(id, self.value.clone());
        (self.func)(State::new(id), cx).dirty(id.child(&0), xform, cx);
    }

    fn hittest(&self, id: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        cx.set_state(id, self.value.clone());
        (self.func)(State::new(id), cx).hittest(id.child(&0), pt, cx)
    }

    fn commands(&self, id: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        cx.set_state(id, self.value.clone());
        (self.func)(State::new(id), cx).commands(id.child(&0), cx, cmds);
    }

    fn gc(&self, id: ViewId, cx: &mut Context, map: &mut Vec<ViewId>) {
        cx.set_state(id, self.value.clone());
        map.push(id);
        (self.func)(State::new(id), cx).gc(id.child(&0), cx, map);
    }

    fn access(
        &self,
        id: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        cx.set_state(id, self.value.clone());
        (self.func)(State::new(id), cx).access(id.child(&0), cx, nodes)
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
