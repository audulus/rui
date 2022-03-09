pub use crate::*;

pub struct Command<V: View, F: Fn()> {
    child: V,
    name: String,
    key: Option<KeyCode>,
    func: F,
}

impl<V, F> Command<V, F>
where
    V: View,
    F: Fn() + 'static,
{
    pub fn new(v: V, name: String, key: Option<KeyCode>, f: F) -> Self {
        Self { child: v, name, key, func: f }
    }
}

impl<V, F> View for Command<V, F>
where
    V: View,
    F: Fn() + 'static,
{
    fn print(&self, id: ViewID, cx: &mut Context) {
        println!("Command {{");
        (self.child).print(id.child(&0), cx);
        println!("}}");
    }

    fn needs_redraw(&self, id: ViewID, cx: &mut Context) -> bool {
        self.child.needs_redraw(id.child(&0), cx)
    }

    fn process(&self, event: &Event, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        if let EventKind::Command(name) = &event.kind {
            if *name == self.name {
                (self.func)();
            }
        }
        self.child.process(event, id.child(&0), cx, vger)
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

    fn commands(&self, id: ViewID, cx: &mut Context, cmds: &mut Vec<String>) {
        self.child.commands(id.child(&0), cx, cmds);
        cmds.push(self.name.clone())
    }
}