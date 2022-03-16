pub use crate::*;

pub struct Focus<V: View, F: Fn(bool) -> V> {
    func: F
}

impl<V, F> View for Focus<V, F> where V: View, F: Fn(bool) -> V {

    fn print(&self, id: ViewID, cx: &mut Context) {
        println!("focus(");
        (self.func)(Some(id) == cx.focused_id).print(id.child(&0), cx);
        println!(")");
    }

    fn process(&self, event: &Event, vid: ViewID, cx: &mut Context, vger: &mut VGER) {
        match &event.kind {
            EventKind::TouchBegin { .. } => {
                if let Some(_) = self.hittest(vid, event.position, cx, vger) {
                    cx.focused_id = Some(vid);
                }
            }
            _ => (),
        }
        (self.func)(Some(vid) == cx.focused_id).process(event, vid.child(&0), cx, vger)
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        (self.func)(Some(id) == cx.focused_id).draw(id.child(&0), cx, vger)
    }

    fn layout(
        &self,
        id: ViewID,
        sz: LocalSize,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> LocalSize {
        (self.func)(Some(id) == cx.focused_id).layout(id.child(&0), sz, cx, vger)
    }

    fn hittest(
        &self,
        id: ViewID,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewID> {
        (self.func)(Some(id) == cx.focused_id).hittest(id.child(&0), pt, cx, vger)
    }

    fn commands(&self, id: ViewID, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        (self.func)(Some(id) == cx.focused_id).commands(id.child(&0), cx, cmds)
    }

}

pub fn focus<V: View, F: Fn(bool) -> V + 'static>(
    f: F,
) -> Focus<V, F> {
    Focus {
        func: f,
    }
}
