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
        path: &mut IdPath,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        path.push(0);
        self.child.process(event, path, cx, actions);
        path.pop();
    }

    fn draw(&self, path: &mut IdPath, args: &mut DrawArgs) {
        let rect = args.cx.get_layout(path).rect;
        (self.func)(args.cx, rect.size, args.vger.current_transform());
        path.push(0);
        self.child.draw(path, args);
        path.pop();
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        path.push(0);
        let sz = self.child.layout(path, args);
        path.pop();

        args.cx.update_layout(
            path,
            LayoutBox {
                rect: LocalRect::new(LocalPoint::zero(), sz),
                offset: LocalOffset::zero(),
            },
        );

        sz
    }

    fn dirty(&self, path: &mut IdPath, xform: LocalToWorld, cx: &mut Context) {
        path.push(0);
        self.child.dirty(path, xform, cx);
        path.pop();
    }

    fn hittest(&self, path: &mut IdPath, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        path.push(0);
        let id = self.child.hittest(path, pt, cx);
        path.pop();
        id
    }

    fn commands(&self, path: &mut IdPath, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        path.push(0);
        self.child.commands(path, cx, cmds);
        path.pop();
    }

    fn gc(&self, path: &mut IdPath, cx: &mut Context, map: &mut Vec<ViewId>) {
        map.push(cx.view_id(path));
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
