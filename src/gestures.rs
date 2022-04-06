pub use crate::*;

/// Struct for the `tap` gesture.
pub struct Tap<V, F> {
    child: V,
    func: F,
}

impl<V, F> Tap<V, F>
where
    V: View,
    F: Fn() + 'static,
{
    pub fn new(v: V, f: F) -> Self {
        Self { child: v, func: f }
    }
}

impl<V, F> View for Tap<V, F>
where
    V: View,
    F: Fn() + 'static,
{
    fn print(&self, id: ViewID, cx: &mut Context) {
        println!("Tap {{");
        (self.child).print(id.child(&0), cx);
        println!("}}");
    }

    fn process(&self, event: &Event, vid: ViewID, cx: &mut Context, vger: &mut VGER) {
        match &event.kind {
            EventKind::TouchBegin { id } => {
                if let Some(_) = self.hittest(vid, event.position, cx, vger) {
                    cx.touches[*id] = vid;
                }
            }
            EventKind::TouchEnd { id } => {
                if cx.touches[*id] == vid {
                    cx.touches[*id] = ViewID::default();
                    (self.func)();
                }
            }
            _ => (),
        }
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        self.child.draw(id.child(&0), cx, vger)
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        self.child.layout(id.child(&0), sz, cx, vger)
    }

    fn hittest(
        &self,
        id: ViewID,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewID> {
        self.child.hittest(id.child(&0), pt, cx, vger)
    }

    fn commands(&self, id: ViewID, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        self.child.commands(id.child(&0), cx, cmds)
    }

    fn gc(&self, id: ViewID, cx: &mut Context, map: &mut StateMap) {
        self.child.gc(id.child(&0), cx, map)
    }

    fn access(
        &self,
        id: ViewID,
        cx: &mut Context,
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        self.child.access(id.child(&0), cx, nodes)
    }
}

impl<V, F> crate::view::private::Sealed for Tap<V, F>
where
    V: View,
    F: Fn() + 'static,
{
}

pub enum GestureState {
    Began,
    Changed,
    Ended,
}

/// Struct for the `drag` gesture.
pub struct Drag<V: View, F: Fn(LocalOffset, GestureState)> {
    child: V,
    func: F,
}

impl<V, F> Drag<V, F>
where
    V: View,
    F: Fn(LocalOffset, GestureState) + 'static,
{
    pub fn new(v: V, f: F) -> Self {
        Self { child: v, func: f }
    }
}

impl<V, F> View for Drag<V, F>
where
    V: View,
    F: Fn(LocalOffset, GestureState) + 'static,
{
    fn print(&self, id: ViewID, cx: &mut Context) {
        println!("Drag {{");
        (self.child).print(id.child(&0), cx);
        println!("}}");
    }

    fn process(&self, event: &Event, vid: ViewID, cx: &mut Context, vger: &mut VGER) {
        match &event.kind {
            EventKind::TouchBegin { id } => {
                if let Some(_) = self.hittest(vid, event.position, cx, vger) {
                    cx.touches[*id] = vid;
                    cx.starts[*id] = event.position;
                    cx.previous_position[*id] = event.position;
                }
            }
            EventKind::TouchMove { id } => {
                if cx.touches[*id] == vid {
                    let delta = event.position - cx.previous_position[*id];
                    (self.func)(delta, GestureState::Changed);
                    cx.previous_position[*id] = event.position;
                }
            }
            EventKind::TouchEnd { id } => {
                if cx.touches[*id] == vid {
                    cx.touches[*id] = ViewID::default();
                    (self.func)(
                        event.position - cx.previous_position[*id],
                        GestureState::Ended,
                    );
                }
            }
            _ => (),
        }
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        self.child.draw(id.child(&0), cx, vger)
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        self.child.layout(id.child(&0), sz, cx, vger)
    }

    fn hittest(
        &self,
        id: ViewID,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewID> {
        self.child.hittest(id.child(&0), pt, cx, vger)
    }

    fn commands(&self, id: ViewID, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        self.child.commands(id.child(&0), cx, cmds)
    }

    fn gc(&self, id: ViewID, cx: &mut Context, map: &mut StateMap) {
        self.child.gc(id.child(&0), cx, map)
    }

    fn access(
        &self,
        id: ViewID,
        cx: &mut Context,
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        self.child.access(id.child(&0), cx, nodes)
    }
}

impl<V, F> crate::view::private::Sealed for Drag<V, F>
where
    V: View,
    F: Fn(LocalOffset, GestureState) + 'static,
{
}
