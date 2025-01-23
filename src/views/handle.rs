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
        path: &mut IdPath,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let mut child_actions = vec![];
        path.push(0);
        self.child.process(event, path, cx, &mut child_actions);
        path.pop();

        for action in child_actions {
            if let Some(a) = action.downcast_ref::<A>() {
                actions.push(Box::new((self.func)(cx, a)));
            } else {
                actions.push(action);
            }
        }
    }

    fn draw(&self, path: &mut IdPath, args: &mut DrawArgs) {
        path.push(0);
        self.child.draw(path, args);
        path.pop();
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        path.push(0);
        let sz = self.child.layout(path, args);
        path.pop();
        sz
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

impl<V, F, A, A2> private::Sealed for Handle<V, F, A, A2> {}
