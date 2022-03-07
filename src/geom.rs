use crate::*;

pub struct Geom<V: View, F: Fn(LocalSize)> {
    child: V,
    func: F,
}

impl<V, F> View for Geom<V, F>
where
    V: View,
    F: Fn(LocalSize),
{
    fn print(&self, id: ViewID, cx: &mut Context) {
        println!("Geom {{");
        (self.child).print(id.child(&0), cx);
        println!("}}");
    }

    fn needs_redraw(&self, id: ViewID, cx: &mut Context) -> bool {
        self.child.needs_redraw(id.child(&0), cx)
    }

    fn process(&self, event: &Event, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        self.child.process(event, id.child(&0), cx, vger);
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        self.child.draw(id.child(&0), cx, vger);
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        let sz = self.child.layout(id.child(&0), sz, cx, vger);
        (self.func)(sz);
        sz
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
        self.child.commands(id.child(&0), cx, cmds)
    }
}

impl<V, F> Geom<V, F>
where
    V: View + 'static,
    F: Fn(LocalSize) + 'static,
{
    pub fn new(child: V, f: F) -> Self {
        Self {
            child: child,
            func: f,
        }
    }
}
