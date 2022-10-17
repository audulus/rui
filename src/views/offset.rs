use crate::*;
use std::any::Any;

/// Struct for the `offset` modifier.
pub struct Offset<V> {
    child: V,
    offset: LocalOffset,
}

impl<V> View for Offset<V>
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
        self.child
            .process(&event.offset(-self.offset), id.child(&0), cx, actions);
    }

    fn draw(&self, id: ViewId, args: &mut DrawArgs) {
        args.vger.save();
        args.vger.translate(self.offset);
        self.child.draw(id.child(&0), args);
        args.vger.restore();
    }

    fn layout(&self, id: ViewId, args: &mut LayoutArgs) -> LocalSize {
        self.child.layout(id.child(&0), args)
    }

    fn dirty(&self, id: ViewId, xform: LocalToWorld, cx: &mut Context) {
        self.child
            .dirty(id.child(&0), xform.pre_translate(self.offset), cx);
    }

    fn bounds(&self, id: ViewId, xform: LocalToWorld, cx: &mut Context) -> WorldRect {
        self.child
            .bounds(id.child(&0), xform.pre_translate(self.offset), cx)
    }

    fn hittest(&self, id: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        self.child.hittest(id.child(&0), pt - self.offset, cx)
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
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        self.child.access(id.child(&0), cx, nodes)
    }
}

impl<V> Offset<V>
where
    V: View,
{
    pub fn new(child: V, offset: LocalOffset) -> Self {
        Self { child, offset }
    }
}

impl<V> private::Sealed for Offset<V> {}
