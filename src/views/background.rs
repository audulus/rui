use crate::*;
use std::any::Any;

/// Struct for the `background` modifier.
#[derive(Clone)]
pub struct Background<V, BG> {
    child: V,
    background: BG,
}

impl<V, BG> DynView for Background<V, BG>
where
    V: View,
    BG: View + Clone,
{
    fn process(
        &self,
        event: &Event,
        vid: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let cid = cx.child(vid, 0);
        self.child.process(event, cid, cx, actions);
        let cid = cx.child(vid, 1);
        self.background.process(event, cid, cx, actions);
    }

    fn draw(&self, vid: ViewId, args: &mut DrawArgs) {
        let cid = args.cx.child(vid, 1);
        self.background.draw(cid, args);
        let cid = args.cx.child(vid, 0);
        self.child.draw(cid, args);
    }

    fn layout(&self, vid: ViewId, args: &mut LayoutArgs) -> LocalSize {
        let cid = args.cx.child(vid, 0);
        let child_size = self.child.layout(cid, args);
        let cid = args.cx.child(vid, 1);
        self.background.layout(cid, &mut args.size(child_size));
        child_size
    }

    fn dirty(&self, vid: ViewId, xform: LocalToWorld, cx: &mut Context) {
        let cid = cx.child(vid, 0);
        self.child.dirty(cid, xform, cx);
        let cid = cx.child(vid, 1);
        self.background.dirty(cid, xform, cx);
    }

    fn hittest(&self, vid: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let cid = cx.child(vid, 1);

        self.background.hittest(cid, pt, cx)
    }

    fn commands(&self, vid: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        let cid = cx.child(vid, 0);
        self.child.commands(cid, cx, cmds);
        let cid = cx.child(vid, 1);
        self.background.commands(cid, cx, cmds);
    }

    fn access(
        &self,
        vid: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        // XXX: if we were to create a node here, what role would it be?
        //      could print a warning if there is an node produced by background.
        let cid = cx.child(vid, 0);

        self.child.access(cid, cx, nodes)
    }
}

impl<V, BG> Background<V, BG>
where
    V: View,
    BG: View + Clone,
{
    pub fn new(child: V, background: BG) -> Self {
        Self { child, background }
    }
}

impl<V, BG> private::Sealed for Background<V, BG> {}
