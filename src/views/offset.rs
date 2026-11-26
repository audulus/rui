use crate::*;
use std::any::Any;

/// Struct for the `offset` modifier.
#[derive(Clone)]
pub struct Offset<V> {
    child: V,
    offset: LocalOffset,
}

impl<V> DynView for Offset<V>
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
        self.child
            .process(&event.offset(-self.offset), cid, cx, actions);
    }

    fn draw(&self, vid: ViewId, args: &mut DrawArgs) {
        args.vger.save();
        args.vger.translate(self.offset);
        let cid = args.cx.child(vid, 0);
        self.child.draw(cid, args);
        args.vger.restore();
    }

    fn layout(&self, vid: ViewId, args: &mut LayoutArgs) -> LocalSize {
        let cid = args.cx.child(vid, 0);

        self.child.layout(cid, args)
    }

    fn dirty(&self, vid: ViewId, xform: LocalToWorld, cx: &mut Context) {
        let cid = cx.child(vid, 0);
        self.child.dirty(cid, xform.pre_translate(self.offset), cx);
    }

    fn hittest(&self, vid: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let cid = cx.child(vid, 0);

        self.child.hittest(cid, pt - self.offset, cx)
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

impl<V> Offset<V>
where
    V: View,
{
    pub fn new(child: V, offset: LocalOffset) -> Self {
        Self { child, offset }
    }
}

impl<V> private::Sealed for Offset<V> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_preserves_size() {
        let mut cx = Context::new();
        let ui = Offset::new(rectangle(), [10.0, 20.0].into());
        let sz = [100.0, 100.0].into();
        let vid = cx.root_id();
        let result = ui.layout(
            vid,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );
        // Offset doesn't change the layout size
        assert_eq!(result, sz);
    }

    #[test]
    fn test_offset_shifts_hittest() {
        let mut cx = Context::new();
        let ui = Offset::new(rectangle(), [50.0, 50.0].into());
        let sz = [100.0, 100.0].into();
        let vid = cx.root_id();
        ui.layout(
            vid,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );
        // The rectangle is offset by (50,50), so point (25,25) maps to (-25,-25) in child space
        // which is outside the 100x100 rect
        assert!(ui.hittest(vid, [25.0, 25.0].into(), &mut cx).is_none());
        // Point (75,75) maps to (25,25) in child space, which is inside
        assert!(ui.hittest(vid, [75.0, 75.0].into(), &mut cx).is_some());
    }
}
