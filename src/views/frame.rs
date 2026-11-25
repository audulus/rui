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

    fn child_offset(path: &mut IdPath, cx: &mut Context) -> LocalOffset {
        path.push(0);
        let offset = cx.get_layout(path).offset;
        path.pop();
        offset
    }
}

impl<V: View> DynView for Frame<V> {
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let offset = Self::child_offset(path, cx);
        path.push(0);
        self.child
            .process(&event.offset(-offset), path, cx, actions);
        path.pop();
    }

    fn draw(&self, path: &mut IdPath, args: &mut DrawArgs) {
        let offset = Self::child_offset(path, args.cx);
        args.vger.save();
        args.vger.translate(offset);
        path.push(0);
        self.child.draw(path, args);
        path.pop();
        args.vger.restore();
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        let proposed = LocalSize::new(
            self.width.unwrap_or(args.sz.width),
            self.height.unwrap_or(args.sz.height),
        );
        path.push(0);
        let child = self.child.layout(path, &mut args.size(proposed));
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
        args.cx.set_layout_offset(path, offset);
        path.pop();
        size
    }

    fn dirty(&self, path: &mut IdPath, xform: LocalToWorld, cx: &mut Context) {
        let offset = Self::child_offset(path, cx);
        path.push(0);
        self.child.dirty(path, xform.pre_translate(offset), cx);
        path.pop();
    }

    fn hittest(&self, path: &mut IdPath, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let offset = Self::child_offset(path, cx);
        path.push(0);
        let id = self.child.hittest(path, pt - offset, cx);
        path.pop();
        id
    }

    fn commands(&self, path: &mut IdPath, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        path.push(0);
        self.child.commands(path, cx, cmds);
        path.pop();
    }

    fn gc(&self, path: &mut IdPath, cx: &mut Context, map: &mut Vec<ViewId>) {
        path.push(0);
        map.push(cx.view_id(path));
        self.child.gc(path, cx, map);
        path.pop();
    }

    fn access(
        &self,
        path: &mut IdPath,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        path.push(0);
        let node_id = self.child.access(path, cx, nodes);
        path.pop();
        node_id
    }
}

impl<V> private::Sealed for Frame<V> {}

#[cfg(test)]
mod tests {
    use super::*;

    fn layout(view: &impl View, sz: [f32; 2], cx: &mut Context) -> LocalSize {
        let mut path = IdPath::root();
        view.layout(
            &mut path,
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
        assert_eq!(
            cx.get_layout(&IdPath::from(vec![0, 0])).offset,
            [10.0, 0.0].into()
        );
    }
}
