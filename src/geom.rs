use crate::*;

pub struct Geom<V: View> {
    child: V,
    func: Box<dyn Fn(LocalSize)>,
}

impl<V> View for Geom<V>
where
    V: View,
{
    fn print(&self, id: ViewID, cx: &mut Context) {
        println!("Geom {{");
        (self.child).print(id.child(0), cx);
        println!("}}");
    }

    fn process(&self, event: &Event, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        self.child.process(event, id.child(0), cx, vger);
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        self.child.draw(id.child(0), cx, vger);
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        let sz = self.child.layout(id.child(0), sz, cx, vger);
        (*self.func)(sz);
        sz
    }

    fn hittest(
        &self,
        id: ViewID,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewID> {
        self.child.hittest(id.child(0), pt, cx, vger)
    }
}

impl<V> Geom<V>
where
    V: View + 'static,
{
    pub fn new<F: Fn(LocalSize) + 'static>(child: V, f: F) -> Self {
        Self {
            child: child,
            func: Box::new(f),
        }
    }
}
