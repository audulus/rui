use crate::*;
use std::any::Any;

/// Struct for the `padding` modifier.
pub struct Padding<V> {
    child: V,
    padding: f32,
}

impl<V> View for Padding<V>
where
    V: View,
{
    fn process(
        &self,
        event: &Event,
        id: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let off = LocalOffset::new(self.padding, self.padding);
        self.child
            .process(&event.offset(-off), id.child(&0), cx, actions);
    }

    fn draw(&self, id: ViewId, args: &mut DrawArgs) {
        args.vger.save();
        args.vger.translate([self.padding, self.padding]);
        self.child.draw(id.child(&0), args);
        args.vger.restore();
    }

    fn layout(&self, id: ViewId, args: &mut LayoutArgs) -> LocalSize {
        let child_size = self.child.layout(
            id.child(&0),
            &mut args.size(args.sz - [2.0 * self.padding, 2.0 * self.padding].into()),
        );
        child_size + LocalSize::new(2.0 * self.padding, 2.0 * self.padding)
    }

    fn dirty(&self, id: ViewId, xform: LocalToWorld, cx: &mut Context) {
        self.child.dirty(
            id.child(&0),
            xform.pre_translate([self.padding, self.padding].into()),
            cx,
        );
    }

    fn hittest(&self, id: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        self.child.hittest(
            id.child(&0),
            pt - LocalOffset::new(self.padding, self.padding),
            cx,
        )
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
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        self.child.access(id.child(&0), cx, nodes)
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
