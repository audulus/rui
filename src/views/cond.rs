use crate::*;
use std::any::Any;

/// Struct for `cond`
#[derive(Clone)]
pub struct Cond<V0, V1> {
    cond: bool,
    if_true: V0,
    if_false: V1,
}

impl<V0, V1> DynView for Cond<V0, V1>
where
    V0: View,
    V1: View,
{
    fn process(
        &self,
        event: &Event,
        vid: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        if self.cond {
            let cid = cx.sole_child(vid, 0);
            self.if_true.process(event, cid, cx, actions);
        } else {
            let cid = cx.sole_child(vid, 1);
            self.if_false.process(event, cid, cx, actions);
        }
    }

    fn draw(&self, vid: ViewId, args: &mut DrawArgs) {
        if self.cond {
            let cid = args.cx.sole_child(vid, 0);
            self.if_true.draw(cid, args);
        } else {
            let cid = args.cx.sole_child(vid, 1);
            self.if_false.draw(cid, args);
        }
    }

    fn layout(&self, vid: ViewId, args: &mut LayoutArgs) -> LocalSize {
        if self.cond {
            let cid = args.cx.sole_child(vid, 0);

            self.if_true.layout(cid, args)
        } else {
            let cid = args.cx.sole_child(vid, 1);

            self.if_false.layout(cid, args)
        }
    }

    fn hittest(&self, vid: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        if self.cond {
            let cid = cx.sole_child(vid, 0);

            self.if_true.hittest(cid, pt, cx)
        } else {
            let cid = cx.sole_child(vid, 1);

            self.if_false.hittest(cid, pt, cx)
        }
    }

    fn commands(&self, vid: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        if self.cond {
            let cid = cx.sole_child(vid, 0);
            self.if_true.commands(cid, cx, cmds);
        } else {
            let cid = cx.sole_child(vid, 1);
            self.if_false.commands(cid, cx, cmds);
        }
    }

    fn access(
        &self,
        vid: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        if self.cond {
            let cid = cx.sole_child(vid, 0);

            self.if_true.access(cid, cx, nodes)
        } else {
            let cid = cx.sole_child(vid, 1);

            self.if_false.access(cid, cx, nodes)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cond_true_uses_first_view() {
        let mut cx = Context::new();
        let ui = cond(
            true,
            rectangle().size([50.0, 50.0]),
            rectangle().size([100.0, 100.0]),
        );
        let sz = [200.0, 200.0].into();
        let root = cx.root_id();
        let result = ui.layout(
            root,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );
        assert_eq!(result, [50.0, 50.0].into());
    }

    #[test]
    fn test_cond_false_uses_second_view() {
        let mut cx = Context::new();
        let ui = cond(
            false,
            rectangle().size([50.0, 50.0]),
            rectangle().size([100.0, 100.0]),
        );
        let sz = [200.0, 200.0].into();
        let root = cx.root_id();
        let result = ui.layout(
            root,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );
        assert_eq!(result, [100.0, 100.0].into());
    }
}
