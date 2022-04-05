use accesskit::Role;
use crate::*;

/// Struct for the `role` modifier.
pub struct RoleView<V: View> {
    child: V,
    role: Role
}

impl<V> RoleView<V>
where
    V: View
{
    pub fn new(v: V, role: Role) -> Self {
        Self { child: v, role }
    }
}

impl<V> View for RoleView<V>
where
    V: View,
{
    fn print(&self, id: ViewID, cx: &mut Context) {
        (self.child).print(id.child(&0), cx);
        println!(".role()");
    }

    fn process(&self, event: &Event, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        self.child.process(&event, id.child(&0), cx, vger);
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        self.child.draw(id.child(&0), cx, vger);
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        self.child.layout(id.child(&0), sz, cx, vger)
    }

    fn hittest(
        &self,
        id: ViewID,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewID> {
        self.child.hittest(
            id.child(&0),
            pt,
            cx,
            vger,
        )
    }

    fn commands(&self, id: ViewID, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        self.child.commands(id.child(&0), cx, cmds)
    }

    fn gc(&self, id: ViewID, cx: &mut Context, map: &mut StateMap) {
        self.child.gc(id.child(&0), cx, map)
    }

    fn access(&self, id: ViewID, cx: &mut Context, nodes: &mut Vec<accesskit::Node>) -> Option<accesskit::NodeId> {
        let child_aid = self.child.access(id.child(&0), cx, nodes);
        let aid = id.access_id();
        nodes.push(
            accesskit::Node {
                children: match child_aid {
                    Some(cid) => vec![cid],
                    None => vec![]
                },
                ..accesskit::Node::new(aid, self.role)
            }
        );
        Some(aid)
    }
}