use crate::*;
use std::any::Any;

pub struct Clip<V> {
    child: V,
}

impl<V> Clip<V>
where V: View, {
    fn geom(&self, id: ViewId, cx: &mut Context) -> LocalRect {
        cx.layout.entry(id).or_default().rect
    }

    pub fn new(child: V) -> Self {
        Self {
            child,
        }
    }
}

impl<V> View for Clip<V>
where
    V: View,
{
    fn process(
        &self,
        event: &Event,
        id: ViewId,
        cx: &mut Context,
        vger: &mut Vger,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        self.child.process(event, id.child(&0), cx, vger, actions);
    }

    fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut Vger) {
        let rect = self.geom(id, cx);

        vger.save();
        vger.scissor(rect);
        self.child.draw(id.child(&0), cx, vger);
        vger.restore();
    }

    fn layout(&self, id: ViewId, sz: LocalSize, cx: &mut Context, vger: &mut Vger) -> LocalSize {
        self.child.layout(id.child(&0), sz, cx, vger);
        cx.layout.insert(
            id,
            LayoutBox {
                rect: LocalRect::new(LocalPoint::zero(), sz),
                offset: LocalOffset::zero(),
            },
        );
        sz
    }

    fn hittest(
        &self,
        id: ViewId,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut Vger,
    ) -> Option<ViewId> {
        let rect = self.geom(id, cx);

        if rect.contains(pt) {
            // Test against children.
            self.child.hittest(id.child(&0), pt, cx, vger)
        } else {
            None
        }
    }

    fn commands(&self, id: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        self.child.commands(id.child(&0), cx, cmds)
    }

    fn gc(&self, id: ViewId, cx: &mut Context, map: &mut Vec<ViewId>) {
        self.child.gc(id.child(&0), cx, map)
    }

    fn access(
        &self,
        id: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        self.child.access(id.child(&0), cx, nodes)
    }
}

impl<V> private::Sealed for Clip<V> {}
