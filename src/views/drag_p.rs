pub use super::drag::GestureState;
use crate::*;
use std::any::Any;

/// Struct for the `drag_p` gesture.
pub struct DragP<V, F> {
    child: V,
    func: F,
    grab: bool,
}

impl<V, F, A> DragP<V, F>
where
    V: View,
    F: Fn(&mut Context, LocalPoint, Option<MouseButton>) -> A + 'static,
{
    pub fn new(v: V, f: F) -> Self {
        Self {
            child: v,
            func: f,
            grab: false,
        }
    }

    pub fn grab_cursor(v: V, f: F) -> Self {
        Self {
            child: v,
            func: f,
            grab: true,
        }
    }
}

impl<V, F, A> View for DragP<V, F>
where
    V: View,
    F: Fn(&mut Context, LocalPoint, Option<MouseButton>) -> A + 'static,
    A: 'static,
{
    fn process(
        &self,
        event: &Event,
        vid: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        match &event {
            Event::TouchBegin { id, position } => {
                if cx.touches[*id].is_default() && self.hittest(vid, *position, cx).is_some() {
                    cx.touches[*id] = vid;
                    cx.starts[*id] = *position;
                    cx.previous_position[*id] = *position;
                    cx.grab_cursor = self.grab;

                    actions.push(Box::new((self.func)(
                        cx,
                        [0.0, 0.0].into(),
                        cx.mouse_button,
                    )));
                }
            }
            Event::TouchMove {
                id,
                position,
                delta: _,
            } => {
                if cx.touches[*id] == vid {
                    actions.push(Box::new((self.func)(cx, *position, cx.mouse_button)));
                    cx.previous_position[*id] = *position;
                }
            }
            Event::TouchEnd { id, position } => {
                if cx.touches[*id] == vid {
                    cx.touches[*id] = ViewId::default();
                    cx.grab_cursor = false;
                    actions.push(Box::new((self.func)(cx, *position, cx.mouse_button)));
                }
            }
            _ => (),
        }
    }

    fn draw(&self, id: ViewId, args: &mut DrawArgs) {
        self.child.draw(id.child(&0), args)
    }

    fn layout(&self, id: ViewId, args: &mut LayoutArgs) -> LocalSize {
        self.child.layout(id.child(&0), args)
    }

    fn dirty(&self, id: ViewId, xform: LocalToWorld, cx: &mut Context) {
        self.child.dirty(id.child(&0), xform, cx);
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

impl<V, F> private::Sealed for DragP<V, F> {}
