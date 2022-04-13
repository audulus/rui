use crate::*;

/// Struct for the `geom` modifier.
pub struct Geom<V, F> {
    child: V,
    func: F,
}

impl<V, F> View for Geom<V, F>
where
    V: View,
    F: Fn(&mut Context, LocalSize) + 'static,
{
    fn print(&self, id: ViewId, cx: &mut Context) {
        println!("Geom {{");
        (self.child).print(id.child(&0), cx);
        println!("}}");
    }

    fn process(&self, event: &Event, id: ViewId, cx: &mut Context, vger: &mut VGER) {
        self.child.process(event, id.child(&0), cx, vger);
    }

    fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut VGER) {
        self.child.draw(id.child(&0), cx, vger);
    }

    fn layout(&self, id: ViewId, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        let sz = self.child.layout(id.child(&0), sz, cx, vger);
        (self.func)(cx, sz);
        sz
    }

    fn hittest(
        &self,
        id: ViewId,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
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

impl<V, F> private::Sealed for Geom<V, F> {}

impl<V, F> Geom<V, F>
where
    V: View + 'static,
    F: Fn(&mut Context, LocalSize) + 'static,
{
    pub fn new(child: V, f: F) -> Self {
        Self { child, func: f }
    }
}
