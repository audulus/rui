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
        path: &mut IdPath,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        if self.cond {
            path.push(0);
            self.if_true.process(event, path, cx, actions);
            path.pop();
        } else {
            path.push(1);
            self.if_false.process(event, path, cx, actions);
            path.pop();
        }
    }

    fn draw(&self, path: &mut IdPath, args: &mut DrawArgs) {
        if self.cond {
            path.push(0);
            self.if_true.draw(path, args);
            path.pop();
        } else {
            path.push(1);
            self.if_false.draw(path, args);
            path.pop();
        }
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        if self.cond {
            path.push(0);
            let sz = self.if_true.layout(path, args);
            path.pop();
            sz
        } else {
            path.push(1);
            let sz = self.if_false.layout(path, args);
            path.pop();
            sz
        }
    }

    fn hittest(&self, path: &mut IdPath, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        if self.cond {
            path.push(0);
            let id = self.if_true.hittest(path, pt, cx);
            path.pop();
            id
        } else {
            path.push(1);
            let id = self.if_false.hittest(path, pt, cx);
            path.pop();
            id
        }
    }

    fn commands(&self, path: &mut IdPath, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        if self.cond {
            path.push(0);
            self.if_true.commands(path, cx, cmds);
            path.pop();
        } else {
            path.push(1);
            self.if_false.commands(path, cx, cmds);
            path.pop();
        }
    }

    fn gc(&self, path: &mut IdPath, cx: &mut Context, map: &mut Vec<ViewId>) {
        if self.cond {
            path.push(0);
            self.if_true.gc(path, cx, map);
            path.pop();
        } else {
            path.push(1);
            self.if_false.gc(path, cx, map);
            path.pop();
        }
    }

    fn access(
        &self,
        path: &mut IdPath,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        if self.cond {
            path.push(0);
            let node_id = self.if_true.access(path, cx, nodes);
            path.pop();
            node_id
        } else {
            path.push(1);
            let node_id = self.if_false.access(path, cx, nodes);
            path.pop();
            node_id
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
        let mut path = vec![0];
        let result = ui.layout(
            &mut path,
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
        let mut path = vec![0];
        let result = ui.layout(
            &mut path,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );
        assert_eq!(result, [100.0, 100.0].into());
    }
}
