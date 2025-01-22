use crate::*;
use std::any::Any;

/// Struct for the `focus` modifier.
#[derive(Clone)]
pub struct Focus<F> {
    func: F,
}

impl<V, F> View for Focus<F>
where
    V: View,
    F: Fn(bool) -> V + Clone + 'static,
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
            Event::TouchBegin { id: _, position } => {
                if self.hittest(path, *position, cx).is_some() {
                    cx.focused_id = Some(vid);
                    cx.set_dirty();
                }
            }
            Event::Key(Key::Escape) => {
                if cx.focused_id == Some(vid) {
                    cx.focused_id = None;
                    cx.set_dirty();
                }
            }
            _ => (),
        }
        path.push(0);
        (self.func)(Some(vid) == cx.focused_id).process(event, path, cx, actions);
        path.pop();
    }

    fn draw(&self, path: &mut IdPath, args: &mut DrawArgs) {
        let id = args.cx.view_id(path);
        path.push(0);
        (self.func)(Some(id) == args.cx.focused_id).draw(path, args);
        path.pop();
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        let id = args.cx.view_id(path);
        path.push(0);
        let sz = (self.func)(Some(id) == args.cx.focused_id).layout(path, args);
        path.pop();
        sz
    }

    fn dirty(&self, path: &mut IdPath, xform: LocalToWorld, cx: &mut Context) {
        let id = cx.view_id(path);
        path.push(0);
        (self.func)(Some(id) == cx.focused_id).dirty(path, xform, cx);
        path.pop();
    }

    fn hittest(&self, path: &mut IdPath, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let id = cx.view_id(path);
        path.push(0);
        let vid = (self.func)(Some(id) == cx.focused_id).hittest(path, pt, cx);
        path.pop();
        vid
    }

    fn commands(&self, path: &mut IdPath, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        let id = cx.view_id(path);
        path.push(0);
        (self.func)(Some(id) == cx.focused_id).commands(path, cx, cmds);
        path.pop();
    }

    fn gc(&self, path: &mut IdPath, cx: &mut Context, map: &mut Vec<ViewId>) {
        let id = cx.view_id(path);
        path.push(0);
        (self.func)(Some(id) == cx.focused_id).gc(path, cx, map);
        path.pop();
    }

    fn access(
        &self,
        path: &mut IdPath,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        let id = cx.view_id(path);
        path.push(0);
        let node_id = (self.func)(Some(id) == cx.focused_id).access(path, cx, nodes);
        path.pop();
        node_id
    }
}

impl<F> private::Sealed for Focus<F> {}

/// Calls calls a function with true if the view subtree returned
/// by the function has the keyboard focus.
pub fn focus<V: View, F: Fn(bool) -> V + Clone + 'static>(f: F) -> impl View {
    Focus { func: f }
}
