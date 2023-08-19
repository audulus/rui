use crate::*;
use std::any::Any;

/// Struct for the `tap` gesture.
pub struct Tap<V: View, F> {
    /// Child view tree.
    child: V,

    /// Called when a tap occurs.
    func: F,
}

impl<V, F, A> Tap<V, F>
where
    V: View,
    F: Fn(&mut Context) -> A + 'static,
{
    pub fn new(v: V, f: F) -> Self {
        Self { child: v, func: f }
    }
}

impl<V, F, A> View for Tap<V, F>
where
    V: View,
    F: Fn(&mut Context) -> A + 'static,
    A: 'static,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let vid = hash(path);
        match &event {
            Event::TouchBegin { id, position } => {
                if self.hittest(path, *position, cx).is_some() {
                    cx.touches[*id] = vid;
                }
            }
            Event::TouchEnd { id, position: _ } => {
                if cx.touches[*id] == vid {
                    cx.touches[*id] = ViewId::default();
                    actions.push(Box::new((self.func)(cx)))
                }
            }
            _ => (),
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

impl<V, F> private::Sealed for Tap<V, F> where V: View {}

/// Struct for the `tap_a` gesture.
pub struct TapA<V: View, A> {
    /// Child view tree.
    child: V,

    /// Called when a tap occurs.
    action: A,
}

impl<V, A> TapA<V, A>
where
    V: View,
{
    pub fn new(child: V, action: A) -> Self {
        Self { child, action }
    }
}

impl<V, A> View for TapA<V, A>
where
    V: View,
    A: Clone + 'static,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let vid = hash(path);
        match &event {
            Event::TouchBegin { id, position } => {
                if self.hittest(path, *position, cx).is_some() {
                    cx.touches[*id] = vid;
                }
            }
            Event::TouchEnd { id, position: _ } => {
                if cx.touches[*id] == vid {
                    cx.touches[*id] = ViewId::default();
                    actions.push(Box::new(self.action.clone()))
                }
            }
            _ => (),
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

impl<V, F> private::Sealed for TapA<V, F> where V: View {}
