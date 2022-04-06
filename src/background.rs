use crate::*;

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
    fn print(&self, id: ViewID, cx: &mut Context) {
        println!("Background {{");
        (self.child).print(id.child(&0), cx);
        (self.background).print(id.child(&1), cx);
        println!("}}");
    }

    fn process(&self, event: &Event, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        self.child.process(event, id.child(&0), cx, vger);
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        self.background.draw(id.child(&1), cx, vger);
        self.child.draw(id.child(&0), cx, vger);
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        let child_size = self.child.layout(id.child(&0), sz, cx, vger);
        self.background.layout(id.child(&1), child_size, cx, vger);
        child_size
    }

    fn hittest(
        &self,
        id: ViewID,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewID> {
        self.background.hittest(id.child(&1), pt, cx, vger)
    }

    fn commands(&self, id: ViewID, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        self.child.commands(id.child(&0), cx, cmds);
        self.background.commands(id.child(&1), cx, cmds);
    }

    fn gc(&self, id: ViewID, cx: &mut Context, map: &mut StateMap) {
        self.child.gc(id.child(&0), cx, map);
        self.background.gc(id.child(&1), cx, map);
    }

    fn access(
        &self,
        id: ViewID,
        cx: &mut Context,
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        // XXX: if we were to create a node here, what role would it be?
        //      could print a warning if there is an node produced by background.
        self.child.access(id.child(&0), cx, nodes)
    }
}

impl<V, BG> Background<V, BG>
where
    V: View + 'static,
    BG: View + 'static,
{
    pub fn new(child: V, background: BG) -> Self {
        Self { child, background }
    }
}

impl<V, BG> crate::private::Sealed for Background<V, BG> {}
