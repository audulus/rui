use crate::*;
use std::any::Any;

/// Struct for the `geom` modifier.
#[derive(Clone)]
pub struct Geom<V, F> {
    child: V,
    func: F,
}

impl<V, F> DynView for Geom<V, F>
where
    V: View,
    F: Fn(&mut Context, LocalSize, LocalToWorld) + Clone + 'static,
{
    fn process(
        &self,
        event: &Event,
        vid: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let cid = cx.child(vid, 0);
        self.child.process(event, cid, cx, actions);
    }

    fn draw(&self, vid: ViewId, args: &mut DrawArgs) {
        let rect = args.cx.get_layout(vid).rect;
        (self.func)(args.cx, rect.size, args.vger.current_transform());
        let cid = args.cx.child(vid, 0);
        self.child.draw(cid, args);
    }

    fn layout(&self, vid: ViewId, args: &mut LayoutArgs) -> LocalSize {
        let cid = args.cx.child(vid, 0);
        let sz = self.child.layout(cid, args);

        args.cx.update_layout(
            vid,
            LayoutBox {
                rect: LocalRect::new(LocalPoint::zero(), sz),
                offset: LocalOffset::zero(),
            },
        );

        sz
    }

    fn dirty(&self, vid: ViewId, xform: LocalToWorld, cx: &mut Context) {
        let cid = cx.child(vid, 0);
        self.child.dirty(cid, xform, cx);
    }

    fn hittest(&self, vid: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let cid = cx.child(vid, 0);

        self.child.hittest(cid, pt, cx)
    }

    fn commands(&self, vid: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        let cid = cx.child(vid, 0);
        self.child.commands(cid, cx, cmds);
    }

    fn access(
        &self,
        vid: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        let cid = cx.child(vid, 0);

        self.child.access(cid, cx, nodes)
    }
}

impl<V, F> private::Sealed for Geom<V, F> {}

impl<V, F> Geom<V, F>
where
    V: View,
    F: Fn(&mut Context, LocalSize, LocalToWorld) + Clone + 'static,
{
    pub fn new(child: V, f: F) -> Self {
        Self { child, func: f }
    }
}
