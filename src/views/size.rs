use crate::*;

/// Struct for the `size` modifier.
pub struct Size<V> {
    child: V,
    size: LocalSize,
}

impl<V> View for Size<V>
where
    V: View,
{
    fn print(&self, id: ViewId, cx: &mut Context) {
        println!("Size {{");
        self.child.print(id.child(&0), cx);
        println!("}}");
    }

    fn process(&self, event: &Event, id: ViewId, cx: &mut Context, vger: &mut Vger) {
        self.child.process(event, id.child(&0), cx, vger);
    }

    fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut Vger) {
        self.child.draw(id.child(&0), cx, vger);
    }

    fn layout(&self, id: ViewId, _sz: LocalSize, cx: &mut Context, vger: &mut Vger) -> LocalSize {
        self.child.layout(id.child(&0), self.size, cx, vger);
        self.size
    }

    fn dirty(
        &self,
        id: ViewId,
        xform: LocalToWorld,
        cx: &mut Context,
    ) {
        self.child.dirty(id.child(&0), xform, cx);
    }

    fn hittest(
        &self,
        id: ViewId,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut Vger,
    ) -> Option<ViewId> {
        self.child.hittest(id.child(&0), pt, cx, vger)
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

impl<V> private::Sealed for Size<V> {}

impl<V> Size<V>
where
    V: View,
{
    pub fn new(child: V, size: LocalSize) -> Self {
        Self { child, size }
    }
}
