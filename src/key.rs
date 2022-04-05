pub use crate::*;

/// Struct for the `key` modifier.
pub struct Key<V: View, F: Fn(KeyPress)> {
    child: V,
    func: F,
}

impl<V, F> Key<V, F>
where
    V: View,
    F: Fn(KeyPress) + 'static,
{
    pub fn new(v: V, f: F) -> Self {
        Self { child: v, func: f }
    }
}

impl<V, F> View for Key<V, F>
where
    V: View,
    F: Fn(KeyPress) + 'static,
{
    fn print(&self, id: ViewID, cx: &mut Context) {
        println!("Key {{");
        (self.child).print(id.child(&0), cx);
        println!("}}");
    }

    fn process(&self, event: &Event, _vid: ViewID, _cx: &mut Context, _vger: &mut VGER) {
        match &event.kind {
            EventKind::Key(key, _) => (self.func)(key.clone()),
            _ => (),
        }
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        self.child.draw(id.child(&0), cx, vger)
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        self.child.layout(id.child(&0), sz, cx, vger)
    }

    fn hittest(
        &self,
        id: ViewID,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewID> {
        self.child.hittest(id.child(&0), pt, cx, vger)
    }

    fn commands(&self, id: ViewID, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        self.child.commands(id.child(&0), cx, cmds)
    }

    fn gc(&self, id: ViewID, cx: &mut Context, map: &mut StateMap) {
        self.child.gc(id.child(&0), cx, map)
    }

    fn access(&self, id: ViewID, cx: &mut Context, nodes: &mut Vec<accesskit::Node>) -> Option<accesskit::NodeId> {
        self.child.access(id.child(&0), cx, nodes)
    }
}
