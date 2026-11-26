use crate::*;
use accesskit::Role;
use std::any::Any;

/// Struct for the `role` modifier.
#[derive(Clone)]
pub struct RoleView<V> {
    child: V,
    role: Role,
}

impl<V> RoleView<V>
where
    V: View,
{
    pub fn new(v: V, role: Role) -> Self {
        Self { child: v, role }
    }
}

impl<V> DynView for RoleView<V>
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

        self.child.layout(cid, args)
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
        let child_aid = self.child.access(cid, cx, nodes);
        let aid = vid.access_id();
        let mut builder = accesskit::NodeBuilder::new(self.role);
        builder.set_children(match child_aid {
            Some(cid) => vec![cid],
            None => vec![],
        });
        nodes.push((aid, builder.build()));
        Some(aid)
    }
}

impl<V> private::Sealed for RoleView<V> {}
