use crate::*;

pub enum GestureState {
    Began,
    Changed,
    Ended,
}

/// Struct for the `drag` gesture.
pub struct Drag<V, F> {
    child: V,
    func: F,
}

impl<V, F> Drag<V, F>
where
    V: View,
    F: Fn(&mut Context, LocalOffset, GestureState, ModifiersState) + 'static,
{
    pub fn new(v: V, f: F) -> Self {
        Self { child: v, func: f }
    }
}

impl<V, F> View for Drag<V, F>
where
    V: View,
    F: Fn(&mut Context, LocalOffset, GestureState, ModifiersState) + 'static,
{
    fn print(&self, id: ViewId, cx: &mut Context) {
        println!("Drag {{");
        (self.child).print(id.child(&0), cx);
        println!("}}");
    }

    fn process(&self, event: &Event, vid: ViewId, cx: &mut Context, vger: &mut VGER) {
        match &event.kind {
            EventKind::TouchBegin { id } => {
                if self.hittest(vid, event.position, cx, vger).is_some() {
                    cx.touches[*id] = vid;
                    cx.starts[*id] = event.position;
                    cx.previous_position[*id] = event.position;
                }
            }
            EventKind::TouchMove { id } => {
                if cx.touches[*id] == vid {
                    let delta = event.position - cx.previous_position[*id];
                    (self.func)(cx, delta, GestureState::Changed, cx.key_mods.clone());
                    cx.previous_position[*id] = event.position;
                }
            }
            EventKind::TouchEnd { id } => {
                if cx.touches[*id] == vid {
                    cx.touches[*id] = ViewId::default();
                    (self.func)(
                        cx,
                        event.position - cx.previous_position[*id],
                        GestureState::Ended,
                        cx.key_mods.clone(),
                    );
                }
            }
            _ => (),
        }
    }

    fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut VGER) {
        self.child.draw(id.child(&0), cx, vger)
    }

    fn layout(&self, id: ViewId, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        self.child.layout(id.child(&0), sz, cx, vger)
    }

    fn hittest(
        &self,
        id: ViewId,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewId> {
        self.child.hittest(id.child(&0), pt, cx, vger)
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
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        self.child.access(id.child(&0), cx, nodes)
    }
}

impl<V, F> private::Sealed for Drag<V, F> {}
