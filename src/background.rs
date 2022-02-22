
use crate::*;

pub struct Background<V: View, BG: View> {
    child: V,
    background: BG,
}

impl<V, BG> View for Background<V, BG>
where
    V: View,
    BG: View
{
    fn print(&self, id: ViewID, cx: &mut Context) {
        println!("Background {{");
        (self.child).print(id.child(0), cx);
        (self.background).print(id.child(1), cx);
        println!("}}");
    }

    fn process(&self, event: &Event, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        self.child.process(event, id.child(0), cx, vger);
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        self.background.draw(id.child(1), cx, vger);
        self.child.draw(id.child(0), cx, vger);
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        let child_size = self.child.layout(
            id.child(0),
            sz,
            cx,
            vger,
        );
        self.background.layout(
            id.child(1),
            child_size,
            cx,
            vger,
        );
        child_size
    }

    fn hittest(
        &self,
        id: ViewID,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewID> {
        self.child.hittest(
            id.child(0),
            pt,
            cx,
            vger,
        )
    }
}

impl<V, BG> Background<V, BG>
where
    V: View + 'static,
    BG: View + 'static
{
    pub fn new(child: V, background: BG) -> Self {
        Self {
            child,
            background,
        }
    }
}
