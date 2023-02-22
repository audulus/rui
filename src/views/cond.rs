use crate::*;
use std::any::Any;

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
    fn process(
        &self,
        event: &Event,
        id: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        if self.cond {
            self.if_true.process(event, id.child(&0), cx, actions)
        } else {
            self.if_false.process(event, id.child(&1), cx, actions)
        }
    }

    fn draw(&self, id: ViewId, args: &mut DrawArgs) {
        if self.cond {
            self.if_true.draw(id.child(&0), args)
        } else {
            self.if_false.draw(id.child(&1), args)
        }
    }

    fn layout(&self, id: ViewId, args: &mut LayoutArgs) -> LocalSize {
        if self.cond {
            self.if_true.layout(id.child(&0), args)
        } else {
            self.if_false.layout(id.child(&1), args)
        }
    }

    fn hittest(&self, id: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        if self.cond {
            self.if_true.hittest(id.child(&0), pt, cx)
        } else {
            self.if_false.hittest(id.child(&1), pt, cx)
        }
    }

    fn commands(&self, id: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        if self.cond {
            self.if_true.commands(id.child(&0), cx, cmds)
        } else {
            self.if_false.commands(id.child(&1), cx, cmds)
        }
    }

    fn gc(&self, id: ViewId, cx: &mut Context, map: &mut Vec<ViewId>) {
        if self.cond {
            self.if_true.gc(id.child(&0), cx, map)
        } else {
            self.if_false.gc(id.child(&1), cx, map)
        }
    }

    fn access(
        &self,
        id: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
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
