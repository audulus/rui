use crate::*;

pub struct ModView<S, F> {
    pub func: F,
    pub value: S,
}

impl<S, V, F> View for ModView<S, F>
where
    V: View,
    S: Clone + Default + 'static,
    F: Fn(S, &mut Context) -> V + 'static,
{
    fn print(&self, id: ViewId, cx: &mut Context) {
        (self.func)(self.value.clone(), cx).print(id.child(&0), cx);
    }

    fn process(&self, event: &Event, id: ViewId, cx: &mut Context, vger: &mut VGER) {
        (self.func)(self.value.clone(), cx).process(event, id.child(&0), cx, vger);
    }

    fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut VGER) {
        (self.func)(self.value.clone(), cx).draw(id.child(&0), cx, vger);
    }

    fn layout(&self, id: ViewId, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        let child_size = (self.func)(self.value.clone(), cx).layout(id.child(&0), sz, cx, vger);

        cx.layout.insert(
            id,
            LayoutBox {
                rect: LocalRect::new(LocalPoint::zero(), child_size),
                offset: LocalOffset::zero(),
            },
        );

        child_size
    }

    fn dirty(
        &self,
        id: ViewId,
        xform: LocalToWorld,
        cx: &mut Context,
        region: &mut Region<WorldSpace>,
    ) {
        (self.func)(self.value.clone(), cx).dirty(id.child(&0), xform, cx, region);
    }

    fn hittest(
        &self,
        id: ViewId,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewId> {
        (self.func)(self.value.clone(), cx).hittest(id.child(&0), pt, cx, vger)
    }

    fn commands(&self, id: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        (self.func)(self.value.clone(), cx).commands(id.child(&0), cx, cmds);
    }

    fn gc(&self, id: ViewId, cx: &mut Context, map: &mut Vec<ViewId>) {
        map.push(id);
        (self.func)(self.value.clone(), cx).gc(id.child(&0), cx, map);
    }

    fn access(
        &self,
        id: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        (self.func)(self.value.clone(), cx).access(id.child(&0), cx, nodes)
    }
}

impl<S, F> private::Sealed for ModView<S, F> {}

/// Passes a value to a function. Value can be updated by modifiers.
pub fn modview<
    S: Clone + Default + 'static,
    V: View,
    F: Fn(S, &mut Context) -> V + 'static,
>(
    f: F,
) -> ModView<S, F> {
    ModView {
        func: f,
        value: Default::default(),
    }
}