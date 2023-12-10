use crate::*;
use std::any::Any;

/// Struct for the `tap` gesture.
pub struct TapP<V: View, F> {
    /// Child view tree.
    child: V,

    /// Called when a tap occurs.
    func: F,
}

impl<V, F, A> TapP<V, F>
where
    V: View,
    F: Fn(&mut Context, LocalPoint, Option<MouseButton>) -> A + 'static,
{
    pub fn new(v: V, f: F) -> Self {
        Self { child: v, func: f }
    }
}

impl<V, F, A> View for TapP<V, F>
where
    V: View,
    F: Fn(&mut Context, LocalPoint, Option<MouseButton>) -> A + 'static,
    A: 'static,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let vid = cx.view_id(path);
        match &event {
            Event::TouchBegin { id, position } => {
                if self.hittest(path, *position, cx).is_some() {
                    cx.touches[*id] = vid;
                }
            }
            Event::TouchEnd { id, position } => {
                if cx.touches[*id] == vid {
                    cx.touches[*id] = ViewId::default();
                    actions.push(Box::new((self.func)(cx, *position, cx.mouse_button)))
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

impl<V, F> private::Sealed for TapP<V, F> where V: View {}
