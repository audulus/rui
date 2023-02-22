use crate::*;
use std::any::Any;

/// Struct for the `background` modifier.
pub struct Background<V, BG> {
    child: V,
    background: BG,
}

impl<V, BG> View for Background<V, BG>
where
    V: View,
    BG: View,
{
    fn process(
        &self,
        event: &Event,
        id: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        self.child.process(event, id.child(&0), cx, actions);
        self.background.process(event, id.child(&1), cx, actions);
    }

    fn draw(&self, id: ViewId, args: &mut DrawArgs) {
        self.background.draw(id.child(&1), args);
        self.child.draw(id.child(&0), args);
    }

    fn layout(&self, id: ViewId, args: &mut LayoutArgs) -> LocalSize {
        let child_size = self.child.layout(id.child(&0), args);
        self.background
            .layout(id.child(&1), &mut args.size(child_size));
        child_size
    }

    fn dirty(&self, id: ViewId, xform: LocalToWorld, cx: &mut Context) {
        self.child.dirty(id.child(&0), xform, cx);
        self.background.dirty(id.child(&1), xform, cx);
    }

    fn hittest(&self, id: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        self.background.hittest(id.child(&1), pt, cx)
    }

    fn commands(&self, id: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        self.child.commands(id.child(&0), cx, cmds);
        self.background.commands(id.child(&1), cx, cmds);
    }

    fn gc(&self, id: ViewId, cx: &mut Context, map: &mut Vec<ViewId>) {
        self.child.gc(id.child(&0), cx, map);
        self.background.gc(id.child(&1), cx, map);
    }

    fn access(
        &self,
        id: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        // XXX: if we were to create a node here, what role would it be?
        //      could print a warning if there is an node produced by background.
        self.child.access(id.child(&0), cx, nodes)
    }
}

impl<V, BG> Background<V, BG>
where
    V: View,
    BG: View,
{
    pub fn new(child: V, background: BG) -> Self {
        Self { child, background }
    }
}

impl<V, BG> private::Sealed for Background<V, BG> {}
