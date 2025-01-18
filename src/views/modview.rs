use crate::*;
use std::any::Any;

#[derive(Clone)]
pub struct ModView<S, F> {
    pub func: F,
    pub value: S,
}

impl<S, V, F> View for ModView<S, F>
where
    V: View,
    S: Clone + Default + 'static,
    F: Fn(S, &mut Context) -> V + 'static,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        path.push(0);
        (self.func)(self.value.clone(), cx).process(event, path, cx, actions);
        path.pop();
    }

    fn draw(&self, path: &mut IdPath, args: &mut DrawArgs) {
        path.push(0);
        (self.func)(self.value.clone(), args.cx).draw(path, args);
        path.pop();
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        path.push(0);
        let sz = (self.func)(self.value.clone(), args.cx).layout(path, args);
        path.pop();
        sz
    }

    fn dirty(&self, path: &mut IdPath, xform: LocalToWorld, cx: &mut Context) {
        path.push(0);
        (self.func)(self.value.clone(), cx).dirty(path, xform, cx);
        path.pop();
    }

    fn hittest(&self, path: &mut IdPath, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        path.push(0);
        let hit_id = (self.func)(self.value.clone(), cx).hittest(path, pt, cx);
        path.pop();
        hit_id
    }

    fn commands(&self, path: &mut IdPath, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        path.push(0);
        (self.func)(self.value.clone(), cx).commands(path, cx, cmds);
        path.pop();
    }

    fn gc(&self, path: &mut IdPath, cx: &mut Context, map: &mut Vec<ViewId>) {
        map.push(cx.view_id(path));
        path.push(0);
        (self.func)(self.value.clone(), cx).gc(path, cx, map);
        path.pop();
    }

    fn access(
        &self,
        path: &mut IdPath,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        path.push(0);
        let node_id = (self.func)(self.value.clone(), cx).access(path, cx, nodes);
        path.pop();
        node_id
    }
}

impl<S, F> private::Sealed for ModView<S, F> {}

/// Passes a value to a function. Value can be updated by modifiers.
pub fn modview<S: Clone + Default + 'static, V: View, F: Fn(S, &mut Context) -> V + 'static>(
    f: F,
) -> ModView<S, F> {
    ModView {
        func: f,
        value: Default::default(),
    }
}
