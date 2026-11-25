use crate::*;
use std::any::Any;

/// Struct for the `size` modifier.
#[derive(Clone)]
pub struct Size<V> {
    /// Child view tree.
    child: V,

    /// Size constraint.
    size: LocalSize,
}

impl<V> DynView for Size<V>
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
        let cid = args.cx.child(vid, 0);
        self.child.draw(cid, args);
    }

    fn layout(&self, vid: ViewId, args: &mut LayoutArgs) -> LocalSize {
        let cid = args.cx.child(vid, 0);
        self.child.layout(cid, &mut args.size(self.size));
        self.size
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

impl<V> private::Sealed for Size<V> {}

impl<V> Size<V>
where
    V: View,
{
    pub fn new(child: V, size: LocalSize) -> Self {
        Self { child, size }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_constrains_layout() {
        let mut cx = Context::new();
        let ui = Size::new(rectangle(), [50.0, 30.0].into());
        let sz = [200.0, 200.0].into();
        let vid = cx.root_id();
        let result = ui.layout(
            vid,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );
        assert_eq!(result, [50.0, 30.0].into());
    }

    #[test]
    fn test_size_ignores_parent_size() {
        let mut cx = Context::new();
        let ui = Size::new(rectangle(), [80.0, 40.0].into());
        // Even with a tiny parent, Size returns its fixed size
        let sz = [10.0, 10.0].into();
        let vid = cx.root_id();
        let result = ui.layout(
            vid,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );
        assert_eq!(result, [80.0, 40.0].into());
    }
}
