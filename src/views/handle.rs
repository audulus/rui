use crate::*;
use std::any::Any;

/// Struct for an action handler.
pub struct Handle<V, F, A> {
    child: V,
    func: F,
    phantom_action: std::marker::PhantomData<A>,
}

impl<V, F, A> Handle<V, F, A>
where
    V: View,
    F: Fn(&mut Context, &A) + 'static,
{
    pub fn new(v: V, f: F) -> Self {
        Self {
            child: v,
            func: f,
            phantom_action: Default::default(),
        }
    }
}

impl<V, F, A> View for Handle<V, F, A>
where
    V: View,
    F: Fn(&mut Context, &A) + 'static,
    A: 'static,
{
    fn process(
        &self,
        event: &Event,
        vid: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let mut child_actions = vec![];
        self.child
            .process(event, vid.child(&0), cx, &mut child_actions);

        for action in child_actions {
            if let Some(a) = action.downcast_ref::<A>() {
                (self.func)(cx, a);
            } else {
                actions.push(action);
            }
        }
    }

    fn draw(&self, id: ViewId, args: &mut DrawArgs) {
        self.child.draw(id.child(&0), args)
    }

    fn layout(&self, id: ViewId, args: &mut LayoutArgs) -> LocalSize {
        self.child.layout(id.child(&0), args)
    }

    fn hittest(&self, id: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        self.child.hittest(id.child(&0), pt, cx)
    }

    fn commands(&self, id: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        self.child.commands(id.child(&0), cx, cmds)
    }

    fn gc(&self, id: ViewId, cx: &mut Context, map: &mut Vec<ViewId>) {
        self.child.gc(id.child(&0), cx, map)
    }

    fn access(
        &self,
        id: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        self.child.access(id.child(&0), cx, nodes)
    }
}

impl<V, F, A> private::Sealed for Handle<V, F, A> {}
