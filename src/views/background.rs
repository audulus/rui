use crate::*;
use std::any::Any;

/// Struct for the `background` modifier.
#[derive(Clone)]
pub struct Background<V, BG> {
    child: V,
    background: BG,
}

impl<V, BG> DynView for Background<V, BG>
where
    V: View,
    BG: View + Clone,
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
        path.push(1);
        self.background.process(event, path, cx, actions);
        path.pop();
    }

    fn draw(&self, path: &mut IdPath, args: &mut DrawArgs) {
        path.push(1);
        self.background.draw(path, args);
        path.pop();
        path.push(0);
        self.child.draw(path, args);
        path.pop();
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        path.push(0);
        let child_size = self.child.layout(path, args);
        path.pop();
        path.push(1);
        self.background.layout(path, &mut args.size(child_size));
        path.pop();
        child_size
    }

    fn dirty(&self, path: &mut IdPath, xform: LocalToWorld, cx: &mut Context) {
        path.push(0);
        self.child.dirty(path, xform, cx);
        path.pop();
        path.push(1);
        self.background.dirty(path, xform, cx);
        path.pop();
    }

    fn hittest(&self, path: &mut IdPath, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        path.push(1);
        let vid = self.background.hittest(path, pt, cx);
        path.pop();
        vid
    }

    fn commands(&self, path: &mut IdPath, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        path.push(0);
        self.child.commands(path, cx, cmds);
        path.pop();
        path.push(1);
        self.background.commands(path, cx, cmds);
        path.pop();
    }

    fn gc(&self, path: &mut IdPath, cx: &mut Context, map: &mut Vec<ViewId>) {
        path.push(0);
        self.child.gc(path, cx, map);
        path.pop();
        path.push(1);
        self.background.gc(path, cx, map);
        path.pop();
    }

    fn access(
        &self,
        path: &mut IdPath,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        // XXX: if we were to create a node here, what role would it be?
        //      could print a warning if there is an node produced by background.
        path.push(0);
        let node_id = self.child.access(path, cx, nodes);
        path.pop();
        node_id
    }
}

impl<V, BG> Background<V, BG>
where
    V: View,
    BG: View + Clone,
{
    pub fn new(child: V, background: BG) -> Self {
        Self { child, background }
    }
}

impl<V, BG> private::Sealed for Background<V, BG> {}
