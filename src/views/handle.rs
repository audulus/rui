use crate::*;
use std::any::Any;

/// Struct for an action handler.
pub struct Handle<V, F, A, A2> {
    child: V,
    func: F,
    phantom_action: std::marker::PhantomData<A>,
    phantom_action2: std::marker::PhantomData<A2>,
}

// Perhaps explicit Clone impl will get around A and A2 needing to be clone?
impl<V, F, A, A2> Clone for Handle<V, F, A, A2>
where
    V: View + Clone,
    F: Fn(&mut Context, &A) -> A2 + Clone + 'static,
{
    fn clone(&self) -> Self {
        Handle::new(self.child.clone(), self.func.clone())
    }
}

impl<V, F, A, A2> Handle<V, F, A, A2>
where
    V: View,
    F: Fn(&mut Context, &A) -> A2 + Clone + 'static,
{
    pub fn new(v: V, f: F) -> Self {
        Self {
            child: v,
            func: f,
            phantom_action: Default::default(),
            phantom_action2: Default::default(),
        }
    }
}

impl<V, F, A, A2> DynView for Handle<V, F, A, A2>
where
    V: View,
    F: Fn(&mut Context, &A) -> A2 + Clone + 'static,
    A: 'static,
    A2: 'static,
{
    fn process(
        &self,
        event: &Event,
        vid: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let mut child_actions = vec![];
        let cid = cx.child(vid, 0);
        self.child.process(event, cid, cx, &mut child_actions);

        for action in child_actions {
            if let Some(a) = action.downcast_ref::<A>() {
                actions.push(Box::new((self.func)(cx, a)));
            } else {
                actions.push(action);
            }
        }
    }

    fn draw(&self, vid: ViewId, args: &mut DrawArgs) {
        let cid = args.cx.child(vid, 0);
        self.child.draw(cid, args);
    }

    fn layout(&self, vid: ViewId, args: &mut LayoutArgs) -> LocalSize {
        let cid = args.cx.child(vid, 0);

        self.child.layout(cid, args)
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

        self.child.access(cid, cx, nodes)
    }
}

impl<V, F, A, A2> private::Sealed for Handle<V, F, A, A2> {}
