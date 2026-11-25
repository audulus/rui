use crate::*;
use std::any::Any;

#[derive(Clone)]
pub struct Clip<V> {
    child: V,
}

impl<V> Clip<V>
where
    V: View,
{
    fn geom(&self, vid: ViewId, cx: &mut Context) -> LocalRect {
        cx.get_layout(vid).rect
    }

    pub fn new(child: V) -> Self {
        Self { child }
    }
}

impl<V> DynView for Clip<V>
where
    V: View,
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
        let rect = self.geom(vid, args.cx);

        args.vger.save();
        args.vger.scissor(rect);
        let visible = args.cx.clip_visible(rect, args.vger);
        let cid = args.cx.child(vid, 0);
        self.child.draw(cid, args);
        args.cx.restore_visible(visible);
        args.vger.restore();
    }

    fn layout(&self, vid: ViewId, args: &mut LayoutArgs) -> LocalSize {
        let cid = args.cx.child(vid, 0);
        self.child.layout(cid, args);
        args.cx.update_layout(
            vid,
            LayoutBox {
                rect: LocalRect::new(LocalPoint::zero(), args.sz),
                offset: LocalOffset::zero(),
            },
        );
        // XXX: should this expand to the available space?
        args.sz
    }

    fn hittest(&self, vid: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let rect = self.geom(vid, cx);

        if rect.contains(pt) {
            // Test against children.
            let cid = cx.child(vid, 0);

            self.child.hittest(cid, pt, cx)
        } else {
            None
        }
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

impl<V> private::Sealed for Clip<V> {}
