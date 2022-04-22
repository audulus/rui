use crate::*;

/// Struct for the `tap` gesture.
pub struct Tap<V, F> {
    child: V,
    func: F,
}

impl<V, F> Tap<V, F>
where
    V: View,
    F: Fn(&mut Context, ModifiersState) + 'static,
{
    pub fn new(v: V, f: F) -> Self {
        Self { child: v, func: f }
    }
}

impl<V, F> View for Tap<V, F>
where
    V: View,
    F: Fn(&mut Context, ModifiersState) + 'static,
{
    fn print(&self, id: ViewId, cx: &mut Context) {
        println!("Tap {{");
        (self.child).print(id.child(&0), cx);
        println!("}}");
    }

    fn process(&self, event: &Event, vid: ViewId, cx: &mut Context, vger: &mut Vger) {
        match &event {
            Event::TouchBegin { id, position } => {
                if self.hittest(vid, *position, cx, vger).is_some() {
                    cx.touches[*id] = vid;
                }
            }
            Event::TouchEnd { id, position: _ } => {
                if cx.touches[*id] == vid {
                    cx.touches[*id] = ViewId::default();
                    (self.func)(cx, cx.key_mods);
                }
            }
            _ => (),
        }
    }

    fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut Vger) {
        self.child.draw(id.child(&0), cx, vger)
    }

    fn layout(&self, id: ViewId, sz: LocalSize, cx: &mut Context, vger: &mut Vger) -> LocalSize {
        self.child.layout(id.child(&0), sz, cx, vger)
    }

    fn hittest(
        &self,
        id: ViewId,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut Vger,
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

impl<V, F> private::Sealed for Tap<V, F> {}
