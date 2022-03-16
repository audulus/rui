pub use crate::*;

pub struct Focus<V: View, B: Binding<bool>> {
    child: V,
    binding: B,
}

impl<V, B> View for Focus<V, B> where V: View, B: Binding<bool> {

    fn print(&self, id: ViewID, cx: &mut Context) {
        self.child.print(id.child(&0), cx);
        println!(".focus()");
    }

    fn needs_redraw(&self, id: ViewID, cx: &mut Context) -> bool {
        self.child.needs_redraw(id.child(&0), cx)
    }

    fn process(&self, event: &Event, vid: ViewID, cx: &mut Context, vger: &mut VGER) {
        match &event.kind {
            EventKind::TouchBegin { id } => {
                if let Some(_) = self.hittest(vid, event.position, cx, vger) {
                    cx.focused_id = Some(vid);
                }
            }
            _ => (),
        }
        self.child.process(event, vid.child(&0), cx, vger)
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        self.child.draw(id.child(&0), cx, vger)
    }

    fn layout(
        &self,
        id: ViewID,
        sz: LocalSize,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> LocalSize {
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

}