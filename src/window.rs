use crate::*;

/// Struct for the `window_title` modifier.
pub struct TitleView<V: View> {
    child: V,
    title: String
}

impl<V> TitleView<V>
where
    V: View
{
    pub fn new(v: V, title: String) -> Self {
        Self { child: v, title }
    }
}

impl<V> View for TitleView<V>
where
    V: View,
{
    fn print(&self, id: ViewID, cx: &mut Context) {
        (self.child).print(id.child(&0), cx);
        println!(".window_title()");
    }

    fn process(&self, event: &Event, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        self.child.process(&event, id.child(&0), cx, vger);
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        self.child.draw(id.child(&0), cx, vger);
        if cx.window_title != self.title {
            cx.window_title = self.title.clone();
            cx.window.set_title(&self.title)
        }
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
        self.child.hittest(
            id.child(&0),
            pt,
            cx,
            vger,
        )
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