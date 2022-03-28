use crate::*;

/// Struct for the `offset` modifier.
pub struct Offset<V: View> {
    child: V,
    offset: LocalOffset,
}

impl<V> View for Offset<V>
where
    V: View,
{
    fn print(&self, id: ViewID, cx: &mut Context) {
        println!("Offset {{");
        (self.child).print(id.child(&0), cx);
        println!("}}");
    }

    fn process(&self, event: &Event, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        let mut local_event = event.clone();
        local_event.position -= self.offset;
        self.child.process(&local_event, id.child(&0), cx, vger);
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        vger.save();
        vger.translate(self.offset);
        self.child.draw(id.child(&0), cx, vger);
        vger.restore();
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
        self.child.hittest(id.child(&0), pt - self.offset, cx, vger)
    }

    fn commands(&self, id: ViewID, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        self.child.commands(id.child(&0), cx, cmds)
    }
}

impl<V> Offset<V>
where
    V: View + 'static,
{
    pub fn new(child: V, offset: LocalOffset) -> Self {
        Self {
            child: child,
            offset,
        }
    }
}
