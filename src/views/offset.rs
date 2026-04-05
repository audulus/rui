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
        path: &mut IdPath,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        path.push(0);
        self.child
            .process(&event.offset(-self.offset), path, cx, actions);
        path.pop();
    }

    fn draw(&self, path: &mut IdPath, args: &mut DrawArgs) {
        args.vger.save();
        args.vger.translate(self.offset);
        path.push(0);
        self.child.draw(path, args);
        path.pop();
        args.vger.restore();
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        path.push(0);
        let sz = self.child.layout(path, args);
        path.pop();
        sz
    }

    fn dirty(&self, path: &mut IdPath, xform: LocalToWorld, cx: &mut Context) {
        path.push(0);
        self.child.dirty(path, xform.pre_translate(self.offset), cx);
        path.pop();
    }

    fn hittest(&self, path: &mut IdPath, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        path.push(0);
        let hit_id = self.child.hittest(path, pt - self.offset, cx);
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
        let mut path = vec![0];
        let result = ui.layout(
            &mut path,
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
        let mut path = vec![0];
        ui.layout(
            &mut path,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );
        // The rectangle is offset by (50,50), so point (25,25) maps to (-25,-25) in child space
        // which is outside the 100x100 rect
        assert!(ui.hittest(&mut path, [25.0, 25.0].into(), &mut cx).is_none());
        // Point (75,75) maps to (25,25) in child space, which is inside
        assert!(ui.hittest(&mut path, [75.0, 75.0].into(), &mut cx).is_some());
    }
}
