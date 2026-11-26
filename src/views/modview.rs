use crate::*;
use std::any::Any;
use std::fmt;

#[derive(Clone)]
pub struct ModView<S, F> {
    pub func: F,
    pub value: S,
}

impl<S, F> fmt::Display for ModView<S, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ModView")
    }
}

impl<S, V, F> DynView for ModView<S, F>
where
    V: View,
    S: Clone + Default + 'static,
    F: Fn(S, &mut Context) -> V + Clone + 'static,
{
    fn process(
        &self,
        event: &Event,
        vid: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let cid = cx.child(vid, 0);
        (self.func)(self.value.clone(), cx).process(event, cid, cx, actions);
    }

    fn draw(&self, vid: ViewId, args: &mut DrawArgs) {
        let cid = args.cx.child(vid, 0);
        (self.func)(self.value.clone(), args.cx).draw(cid, args);
    }

    fn layout(&self, vid: ViewId, args: &mut LayoutArgs) -> LocalSize {
        let cid = args.cx.child(vid, 0);

        (self.func)(self.value.clone(), args.cx).layout(cid, args)
    }

    fn dirty(&self, vid: ViewId, xform: LocalToWorld, cx: &mut Context) {
        let cid = cx.child(vid, 0);
        (self.func)(self.value.clone(), cx).dirty(cid, xform, cx);
    }

    fn hittest(&self, vid: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let cid = cx.child(vid, 0);

        (self.func)(self.value.clone(), cx).hittest(cid, pt, cx)
    }

    fn commands(&self, vid: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        let cid = cx.child(vid, 0);
        (self.func)(self.value.clone(), cx).commands(cid, cx, cmds);
    }

    fn access(
        &self,
        vid: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        let cid = cx.child(vid, 0);

        (self.func)(self.value.clone(), cx).access(cid, cx, nodes)
    }
}

/// Passes a value to a function. Value can be updated by modifiers.
pub fn modview<
    S: Clone + Default + 'static,
    V: View,
    F: Fn(S, &mut Context) -> V + Clone + 'static,
>(
    f: F,
) -> ModView<S, F> {
    ModView {
        func: f,
        value: Default::default(),
    }
}
