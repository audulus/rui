use crate::*;
use std::any::Any;

/// Struct for the `fixed_width` and `fixed_height` modifiers.
#[derive(Clone)]
pub struct Frame<V> {
    child: V,
    width: Option<f32>,
    height: Option<f32>,
}

impl<V: View> Frame<V> {
    pub fn new(child: V, width: Option<f32>, height: Option<f32>) -> Self {
        Self {
            child,
            width,
            height,
        }
    }

    fn child_offset(vid: ViewId, cx: &mut Context) -> LocalOffset {
        let cid = cx.child(vid, 0);

        cx.get_layout(cid).offset
    }
}

impl<V: View> DynView for Frame<V> {
    fn process(
        &self,
        event: &Event,
        vid: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let offset = Self::child_offset(vid, cx);
        let cid = cx.child(vid, 0);
        self.child.process(&event.offset(-offset), cid, cx, actions);
    }

    fn draw(&self, vid: ViewId, args: &mut DrawArgs) {
        let offset = Self::child_offset(vid, args.cx);
        args.vger.save();
        args.vger.translate(offset);
        let cid = args.cx.child(vid, 0);
        self.child.draw(cid, args);
        args.vger.restore();
    }

    fn layout(&self, vid: ViewId, args: &mut LayoutArgs) -> LocalSize {
        let proposed = LocalSize::new(
            self.width.unwrap_or(args.sz.width),
            self.height.unwrap_or(args.sz.height),
        );
        let cid = args.cx.child(vid, 0);
        let child = self.child.layout(cid, &mut args.size(proposed));
        let size = LocalSize::new(
            self.width.unwrap_or(child.width),
            self.height.unwrap_or(child.height),
        );
        // Center the child in the frame.
        let offset = align(
            LocalRect::new(LocalPoint::origin(), child),
            LocalRect::new(LocalPoint::origin(), size),
            HAlignment::Center,
            VAlignment::Middle,
        );
        args.cx.set_layout_offset(cid, offset);
        size
    }

    fn dirty(&self, vid: ViewId, xform: LocalToWorld, cx: &mut Context) {
        let offset = Self::child_offset(vid, cx);
        let cid = cx.child(vid, 0);
        self.child.dirty(cid, xform.pre_translate(offset), cx);
    }

    fn hittest(&self, vid: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let offset = Self::child_offset(vid, cx);
        let cid = cx.child(vid, 0);

        self.child.hittest(cid, pt - offset, cx)
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

impl<V> private::Sealed for Frame<V> {}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout(view: &impl View, sz: [f32; 2], cx: &mut Context) -> LocalSize {
        let vid = cx.root_id();
        view.layout(
            vid,
            &mut LayoutArgs {
                sz: sz.into(),
                cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        )
    }

    #[test]
    fn test_fixed_width_keeps_child_height() {
        let mut cx = Context::new();
        let ui = rectangle().size([10.0, 30.0]).fixed_width(100.0);
        assert_eq!(layout(&ui, [200.0, 200.0], &mut cx), [100.0, 30.0].into());
    }

    #[test]
    fn test_fixed_height_takes_offered_width() {
        let mut cx = Context::new();
        let ui = hstack((rectangle().size([10.0, 10.0]), spacer())).fixed_height(40.0);
        assert_eq!(layout(&ui, [200.0, 500.0], &mut cx), [200.0, 40.0].into());
    }

    #[test]
    fn test_frame_centers_child() {
        let mut cx = Context::new();
        let ui = rectangle().size([10.0, 10.0]).fixed_width(30.0);
        layout(&ui, [100.0, 100.0], &mut cx);
        assert_eq!(cx.get_layout(cx.id_at(&[0, 0])).offset, [10.0, 0.0].into());
    }
}
