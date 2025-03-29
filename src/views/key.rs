use crate::*;
use std::any::Any;

/// Describes if the KeyView action should trigger when pressing or releasing a key
#[derive(Clone)]
pub enum KeyViewKind {
    Pressed,
    Released,
}

/// Struct for the `key` modifier.
#[derive(Clone)]
pub struct KeyView<V, F> {
    child: V,
    func: F,
    kind: KeyViewKind,
}

impl<V, F, A> KeyView<V, F>
where
    V: View,
    F: Fn(&mut Context, Key) -> A + Clone + 'static,
{
    pub fn new_pressed(v: V, f: F) -> Self {
        KeyView {
            child: v,
            func: f,
            kind: KeyViewKind::Pressed,
        }
    }

    pub fn new_released(v: V, f: F) -> Self {
        KeyView {
            child: v,
            func: f,
            kind: KeyViewKind::Released,
        }
    }
}

impl<V, F, A> DynView for KeyView<V, F>
where
    V: View,
    F: Fn(&mut Context, Key) -> A + Clone + 'static,
    A: 'static,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        match self.kind {
            KeyViewKind::Pressed => {
                if let Event::Key(key) = &event {
                    actions.push(Box::new((self.func)(cx, *key)));
                } else {
                    path.push(0);
                    self.child.process(event, path, cx, actions);
                    path.pop();
                }
            }
            KeyViewKind::Released => {
                if let Event::KeyReleased(key) = &event {
                    actions.push(Box::new((self.func)(cx, *key)));
                } else {
                    path.push(0);
                    self.child.process(event, path, cx, actions);
                    path.pop();
                }
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

    fn dirty(&self, path: &mut IdPath, xform: LocalToWorld, cx: &mut Context) {
        path.push(0);
        self.child.dirty(path, xform, cx);
        path.pop();
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

impl<V, F> private::Sealed for KeyView<V, F> {}
