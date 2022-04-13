use crate::*;

/// Struct for the `key` modifier.
pub struct Key<V, F> {
    child: V,
    func: F,
}

impl<V, F> Key<V, F>
where
    V: View,
    F: Fn(&mut Context, KeyPress) + 'static,
{
    pub fn new(v: V, f: F) -> Self {
        Self { child: v, func: f }
    }
}

impl<V, F> View for Key<V, F>
where
    V: View,
    F: Fn(&mut Context, KeyPress) + 'static,
{
    fn print(&self, id: ViewId, cx: &mut Context) {
        println!("Key {{");
        (self.child).print(id.child(&0), cx);
        println!("}}");
    }

    fn process(&self, event: &Event, _vid: ViewId, cx: &mut Context, _vger: &mut VGER) {
        if let EventKind::Key(key, _) = &event.kind {
            (self.func)(cx, key.clone())
        }
    }

    fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut VGER) {
        self.child.draw(id.child(&0), cx, vger)
    }

    fn layout(&self, id: ViewId, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        self.child.layout(id.child(&0), sz, cx, vger)
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

impl<V, F> private::Sealed for Key<V, F> {}
