use crate::*;

/// Struct for the `flex` modifier.
pub struct Flex<V> {
    child: V,
}

impl<V> View for Flex<V>
where
    V: View,
{
    fn print(&self, id: ViewId, cx: &mut Context) {
        println!("Flex {{");
        (self.child).print(id.child(&0), cx);
        println!("}}");
    }

    fn process(&self, event: &Event, id: ViewId, cx: &mut Context, vger: &mut VGER) {
        self.child.process(&event, id.child(&0), cx, vger);
    }

    fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut VGER) {
        self.child.draw(id.child(&0), cx, vger);
    }

    fn layout(&self, id: ViewId, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        self.child.layout(
            id.child(&0),
            sz,
            cx,
            vger,
        )
    }

    fn dirty(
        &self,
        id: ViewId,
        xform: LocalToWorld,
        cx: &mut Context,
        region: &mut Region<WorldSpace>,
    ) {
        self.child.dirty(
            id.child(&0),
            xform,
            cx,
            region,
        );
    }

    fn hittest(
        &self,
        id: ViewId,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewId> {
        self.child.hittest(
            id.child(&0),
            pt,
            cx,
            vger,
        )
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