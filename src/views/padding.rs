use crate::*;
use std::any::Any;

/// Struct for the `padding` modifier.
#[derive(Clone)]
pub struct Padding<V> {
    child: V,
    padding: f32,
}

impl<V> DynView for Padding<V>
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
        let off = LocalOffset::new(self.padding, self.padding);
        let cid = cx.child(vid, 0);
        self.child.process(&event.offset(-off), cid, cx, actions);
    }

    fn draw(&self, vid: ViewId, args: &mut DrawArgs) {
        args.vger.save();
        args.vger.translate([self.padding, self.padding]);
        let cid = args.cx.child(vid, 0);
        self.child.draw(cid, args);
        args.vger.restore();
    }

    fn layout(&self, vid: ViewId, args: &mut LayoutArgs) -> LocalSize {
        let cid = args.cx.child(vid, 0);
        let child_size = self.child.layout(
            cid,
            &mut args.size(args.sz - [2.0 * self.padding, 2.0 * self.padding].into()),
        );
        child_size + LocalSize::new(2.0 * self.padding, 2.0 * self.padding)
    }

    fn dirty(&self, vid: ViewId, xform: LocalToWorld, cx: &mut Context) {
        let cid = cx.child(vid, 0);
        self.child.dirty(
            cid,
            xform.pre_translate([self.padding, self.padding].into()),
            cx,
        );
    }

    fn hittest(&self, vid: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let cid = cx.child(vid, 0);

        self.child
            .hittest(cid, pt - LocalOffset::new(self.padding, self.padding), cx)
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

pub enum PaddingParam {
    Auto,
    Px(f32),
}
pub struct Auto;
impl From<Auto> for PaddingParam {
    fn from(_val: Auto) -> Self {
        PaddingParam::Auto
    }
}
impl From<f32> for PaddingParam {
    fn from(val: f32) -> Self {
        PaddingParam::Px(val)
    }
}

impl<V> Padding<V>
where
    V: View,
{
    pub fn new(child: V, param: PaddingParam) -> Self {
        Self {
            child,
            padding: match param {
                PaddingParam::Auto => 5.0,
                PaddingParam::Px(px) => px,
            },
        }
    }
}

impl<V> private::Sealed for Padding<V> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_padding_increases_size() {
        let mut cx = Context::new();
        let ui = Padding::new(rectangle(), PaddingParam::Px(10.0));
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
        // rectangle fills available space (100-20=80), then padding adds 20 back
        assert_eq!(result, [100.0, 100.0].into());
    }

    #[test]
    fn test_padding_auto() {
        let mut cx = Context::new();
        let ui = Padding::new(rectangle(), PaddingParam::Auto);
        let sz = [50.0, 50.0].into();
        let vid = cx.root_id();
        let result = ui.layout(
            vid,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );
        // Auto padding is 5.0, so child gets 40x40, result is 50x50
        assert_eq!(result, [50.0, 50.0].into());
    }

    #[test]
    fn test_padding_hittest_offsets() {
        let mut cx = Context::new();
        let ui = Padding::new(rectangle(), PaddingParam::Px(20.0));
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
        // Point inside the padded child area (child starts at 20,20)
        assert!(ui.hittest(vid, [50.0, 50.0].into(), &mut cx).is_some());
        // Point in the padding area (outside child)
        assert!(ui.hittest(vid, [5.0, 5.0].into(), &mut cx).is_none());
    }
}
