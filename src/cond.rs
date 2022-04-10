use crate::*;

/// Struct for `cond`
pub struct Cond<V0, V1> {
    cond: bool,
    if_true: V0,
    if_false: V1,
}

impl<V0, V1> View for Cond<V0, V1>
where
    V0: View,
    V1: View,
{
    fn print(&self, id: ViewID, cx: &mut Context) {
        if self.cond {
            self.if_true.print(id.child(&0), cx)
        } else {
            self.if_false.print(id.child(&1), cx)
        }
    }

    fn process(&self, event: &Event, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        if self.cond {
            self.if_true.process(event, id.child(&0), cx, vger)
        } else {
            self.if_false.process(event, id.child(&1), cx, vger)
        }
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        if self.cond {
            self.if_true.draw(id.child(&0), cx, vger)
        } else {
            self.if_false.draw(id.child(&0), cx, vger)
        }
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        if self.cond {
            self.if_true.layout(id.child(&0), sz, cx, vger)
        } else {
            self.if_false.layout(id.child(&1), sz, cx, vger)
        }
    }

    fn hittest(
        &self,
        id: ViewID,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewID> {
        if self.cond {
            self.if_true.hittest(id.child(&0), pt, cx, vger)
        } else {
            self.if_false.hittest(id.child(&1), pt, cx, vger)
        }
    }

    fn commands(&self, id: ViewID, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        if self.cond {
            self.if_true.commands(id.child(&0), cx, cmds)
        } else {
            self.if_false.commands(id.child(&1), cx, cmds)
        }
    }

    fn gc(&self, id: ViewID, cx: &mut Context, map: &mut Vec<ViewID>) {
        if self.cond {
            self.if_true.gc(id.child(&0), cx, map)
        } else {
            self.if_false.gc(id.child(&1), cx, map)
        }
    }

    fn access(
        &self,
        id: ViewID,
        cx: &mut Context,
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        if self.cond {
            self.if_true.access(id.child(&0), cx, nodes)
        } else {
            self.if_false.access(id.child(&1), cx, nodes)
        }
    }
}

impl<V0, V1> private::Sealed for Cond<V0, V1> {}

/// Switches between views according to a boolean.
pub fn cond(cond: bool, if_true: impl View, if_false: impl View) -> impl View {
    Cond {
        cond,
        if_true,
        if_false,
    }
}
