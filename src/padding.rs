use crate::*;

pub struct Padding<V: View> {
    child: V,
    padding: f32,
}

impl<V> View for Padding<V>
where
    V: View,
{
    fn print(&self, id: ViewID, cx: &mut Context) {
        println!("Padding {{");
        (self.child).print(id.child(0), cx);
        println!("}}");
    }

    fn process(&self, event: &Event, id: ViewID, cx: &mut Context) {
        self.child.process(event, id.child(0), cx);
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        self.child.draw(id, cx, vger);
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        let child_size = self.child.layout(
            id.child(0),
            sz - [self.padding, self.padding].into(),
            cx,
            vger,
        );
        child_size + LocalSize::new(self.padding, self.padding)
    }

    fn hittest(&self, id: ViewID, pt: LocalPoint, cx: &mut Context, vger: &mut VGER) -> bool {
        self.child.hittest(
            id.child(0),
            pt - LocalOffset::new(self.padding, self.padding),
            cx,
            vger,
        )
    }
}

impl<V> Padding<V>
where
    V: View + 'static,
{
    pub fn new(child: V) -> Self {
        Self {
            child: child,
            padding: 5.0,
        }
    }
}
