use crate::*;
use std::any::Any;

/// Struct for the `on_scroll` gesture.
#[derive(Clone)]
pub struct ScrollGesture<V, F> {
    child: V,
    func: F,
}

impl<V, F> ScrollGesture<V, F> {
    pub fn new(child: V, func: F) -> Self {
        Self { child, func }
    }
}

impl<V, A, F> DynView for ScrollGesture<V, F>
where
    V: View,
    A: 'static,
    F: Fn(&mut Context, LocalOffset) -> A + Clone + 'static,
{
    fn process(
        &self,
        event: &Event,
        vid: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        if let Event::Scroll { position, delta } = &event
            && self.hittest(vid, *position, cx).is_some()
        {
            actions.push(Box::new((self.func)(cx, *delta)));
        }
        let cid = cx.child(vid, 0);
        self.child.process(event, cid, cx, actions);
    }

    fn draw(&self, vid: ViewId, args: &mut DrawArgs) {
        let cid = args.cx.child(vid, 0);
        self.child.draw(cid, args);
    }

    fn layout(&self, vid: ViewId, args: &mut LayoutArgs) -> LocalSize {
        let cid = args.cx.child(vid, 0);

        self.child.layout(cid, args)
    }

    fn dirty(&self, vid: ViewId, xform: LocalToWorld, cx: &mut Context) {
        let cid = cx.child(vid, 0);
        self.child.dirty(cid, xform, cx);
    }

    fn hittest(&self, vid: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let cid = cx.child(vid, 0);

        self.child.hittest(cid, pt, cx)
    }

    fn commands(&self, vid: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        let cid = cx.child(vid, 0);
        self.child.commands(cid, cx, cmds);
    }

    fn access(
        &self,
        vid: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        let cid = cx.child(vid, 0);

        self.child.access(cid, cx, nodes)
    }
}

impl<V, F> private::Sealed for ScrollGesture<V, F> {}
