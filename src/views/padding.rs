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
        path: &mut IdPath,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let off = LocalOffset::new(self.padding, self.padding);
        path.push(0);
        self.child.process(&event.offset(-off), path, cx, actions);
        path.pop();
    }

    fn draw(&self, path: &mut IdPath, args: &mut DrawArgs) {
        args.vger.save();
        args.vger.translate([self.padding, self.padding]);
        path.push(0);
        self.child.draw(path, args);
        path.pop();
        args.vger.restore();
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        path.push(0);
        let child_size = self.child.layout(
            path,
            &mut args.size(args.sz - [2.0 * self.padding, 2.0 * self.padding].into()),
        );
        path.pop();
        child_size + LocalSize::new(2.0 * self.padding, 2.0 * self.padding)
    }

    fn dirty(&self, path: &mut IdPath, xform: LocalToWorld, cx: &mut Context) {
        path.push(0);
        self.child.dirty(
            path,
            xform.pre_translate([self.padding, self.padding].into()),
            cx,
        );
        path.pop();
    }

    fn hittest(&self, path: &mut IdPath, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        path.push(0);
        let hit_id =
            self.child
                .hittest(path, pt - LocalOffset::new(self.padding, self.padding), cx);
        path.pop();
        hit_id
    }

    fn commands(&self, path: &mut IdPath, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        path.push(0);
        self.child.commands(path, cx, cmds);
        path.pop();
    }

    fn gc(&self, path: &mut IdPath, cx: &mut Context, map: &mut Vec<ViewId>) {
        path.push(0);
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
        let mut path = vec![0];
        let result = ui.layout(
            &mut path,
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
        let mut path = vec![0];
        let result = ui.layout(
            &mut path,
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
        let mut path = vec![0];
        ui.layout(
            &mut path,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );
        // Point inside the padded child area (child starts at 20,20)
        assert!(ui.hittest(&mut path, [50.0, 50.0].into(), &mut cx).is_some());
        // Point in the padding area (outside child)
        assert!(ui.hittest(&mut path, [5.0, 5.0].into(), &mut cx).is_none());
    }
}
