pub use crate::*;

pub struct Key<V: View, F: Fn(KeyCode)> {
    child: V,
    func: F,
}

impl<V, F> Key<V, F>
where
    V: View,
    F: Fn(KeyCode) + 'static,
{
    pub fn new(v: V, f: F) -> Self {
        Self { child: v, func: f }
    }
}

impl<V, F> View for Key<V, F>
where
    V: View,
    F: Fn(KeyCode) + 'static,
{
    fn print(&self, id: ViewID, cx: &mut Context) {
        println!("Key {{");
        (self.child).print(id.child(&0), cx);
        println!("}}");
    }

    fn needs_redraw(&self, id: ViewID, cx: &mut Context) -> bool {
        self.child.needs_redraw(id.child(&0), cx)
    }

    fn process(&self, event: &Event, _vid: ViewID, _cx: &mut Context, _vger: &mut VGER) {
        match &event.kind {
            EventKind::Key(code) => {
                (self.func)(*code)
            }
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
}