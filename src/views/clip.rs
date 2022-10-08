use crate::*;
use std::any::Any;

pub struct Clip<V> {
    child: V,
}

impl<V> Clip<V>
where
    V: View,
{
    fn geom(&self, id: ViewId, cx: &mut Context) -> LocalRect {
        cx.layout.entry(id).or_default().rect
    }

    pub fn new(child: V) -> Self {
        Self { child }
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

    fn draw(&self, id: ViewId, args: &mut DrawArgs) {
        let rect = self.geom(id, args.cx);

        args.vger.save();
        args.vger.scissor(rect);
        self.child.draw(id.child(&0), args);
        args.vger.restore();
    }

    fn layout(&self, id: ViewId, args: &mut LayoutArgs) -> LocalSize {
        self.child.layout(id.child(&0), args);
        args.cx.layout.insert(
            id,
            LayoutBox {
                rect: LocalRect::new(LocalPoint::zero(), args.sz),
                offset: LocalOffset::zero(),
            },
        );
        // XXX: should this expand to the available space?
        args.sz
    }

    fn hittest(
        &self,
        id: ViewId,
        pt: LocalPoint,
        cx: &mut Context,
    ) -> Option<ViewId> {
        let rect = self.geom(id, cx);

        if rect.contains(pt) {
            // Test against children.
            self.child.hittest(id.child(&0), pt, cx)
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
