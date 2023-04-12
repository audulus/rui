use crate::*;
use std::any::Any;

/// Struct for the `geom` modifier.
pub struct Geom<V, F> {
    child: V,
    func: F,
}

impl<V, F> View for Geom<V, F>
where
    V: View,
    F: Fn(&mut Context, LocalSize, LocalToWorld) + 'static,
{
    fn process(
        &self,
        event: &Event,
        id: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        self.child.process(event, id.child(&0), cx, actions);
    }

    fn draw(&self, id: ViewId, args: &mut DrawArgs) {
        let rect = args.cx.layout[&id].rect;
        (self.func)(args.cx, rect.size, args.vger.current_transform());
        self.child.draw(id.child(&0), args);
    }

    fn layout(&self, id: ViewId, args: &mut LayoutArgs) -> LocalSize {
        let sz = self.child.layout(id.child(&0), args);

        args.cx.layout.insert(
            id,
            LayoutBox {
                rect: LocalRect::new(LocalPoint::zero(), sz),
                offset: LocalOffset::zero(),
            },
        );

        sz
    }

    fn dirty(&self, id: ViewId, xform: LocalToWorld, cx: &mut Context) {
        self.child.dirty(id.child(&0), xform, cx);
    }

    fn hittest(&self, id: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        self.child.hittest(id.child(&0), pt, cx)
    }

    fn commands(&self, id: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        self.child.commands(id.child(&0), cx, cmds)
    }

    fn gc(&self, id: ViewId, cx: &mut Context, map: &mut Vec<ViewId>) {
        map.push(id);
        self.child.gc(id.child(&0), cx, map)
    }

    fn access(
        &self,
        id: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        self.child.access(id.child(&0), cx, nodes)
    }
}

impl<V, F> private::Sealed for Geom<V, F> {}

impl<V, F> Geom<V, F>
where
    V: View,
    F: Fn(&mut Context, LocalSize, LocalToWorld) + 'static,
{
    pub fn new(child: V, f: F) -> Self {
        Self { child, func: f }
    }
}
