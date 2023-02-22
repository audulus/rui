use crate::*;
use std::any::Any;

/// Struct for the `flex` modifier.
pub struct Flex<V> {
    child: V,
}

impl<V> View for Flex<V>
where
    V: View,
{
    fn process(
        &self,
        event: &Event,
        id: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        self.child.process(event, id.child(&0), cx, actions);
    }

    fn draw(&self, id: ViewId, args: &mut DrawArgs) {
        self.child.draw(id.child(&0), args);
    }

    fn layout(&self, id: ViewId, args: &mut LayoutArgs) -> LocalSize {
        self.child.layout(id.child(&0), args)
    }

    fn dirty(&self, id: ViewId, xform: LocalToWorld, cx: &mut Context) {
        self.child.dirty(id.child(&0), xform, cx);
    }

    fn hittest(&self, id: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        self.child.hittest(id.child(&0), pt, cx)
    }

    fn commands(&self, id: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        self.child.commands(id.child(&0), cx, cmds)
    }

    fn gc(&self, id: ViewId, cx: &mut Context, map: &mut Vec<ViewId>) {
        self.child.gc(id.child(&0), cx, map)
    }

    fn access(
        &self,
        id: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        self.child.access(id.child(&0), cx, nodes)
    }

    fn is_flexible(&self) -> bool {
        true
    }
}

impl<V: View> Flex<V> {
    pub fn new(child: V) -> Self {
        Self { child }
    }
}

impl<V> private::Sealed for Flex<V> {}
