use crate::*;

// Using this depends on https://github.com/rust-lang/rust/issues/63063
pub trait Body: View {
    type V: View;
    fn body(&self) -> Self::V;

    fn print(&self, id: ViewId, cx: &mut Context) {
        self.body().print(id, cx)
    }

    fn process(&self, event: &Event, id: ViewId, cx: &mut Context, vger: &mut VGER) {
        self.body().process(event, id, cx, vger)
    }

    fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut VGER) {
        self.body().draw(id, cx, vger)
    }

    fn layout(&self, id: ViewId, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        self.body().layout(id, sz, cx, vger)
    }

    fn hittest(
        &self,
        id: ViewId,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewId> {
        self.body().hittest(id, pt, cx, vger)
    }

    fn commands(&self, id: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        self.body().commands(id, cx, cmds);
    }
}

#[macro_export]
macro_rules! body_view {
    () => {
        fn print(&self, id: ViewId, cx: &mut Context) {
            self.body().print(id, cx)
        }

        fn process(&self, event: &Event, id: ViewId, cx: &mut Context, vger: &mut VGER) {
            self.body().process(event, id, cx, vger)
        }

        fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut VGER) {
            self.body().draw(id, cx, vger)
        }

        fn layout(
            &self,
            id: ViewId,
            sz: LocalSize,
            cx: &mut Context,
            vger: &mut VGER,
        ) -> LocalSize {
            self.body().layout(id, sz, cx, vger)
        }

        fn hittest(
            &self,
            id: ViewId,
            pt: LocalPoint,
            cx: &mut Context,
            vger: &mut VGER,
        ) -> Option<ViewId> {
            self.body().hittest(id, pt, cx, vger)
        }

        fn commands(&self, id: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
            self.body().commands(id, cx, cmds);
        }

        fn gc(&self, id: ViewId, cx: &mut Context, map: &mut Vec<ViewId>) {
            self.body().gc(id, cx, map)
        }

        fn access(
            &self,
            id: ViewId,
            cx: &mut Context,
            nodes: &mut Vec<accesskit::Node>,
        ) -> Option<accesskit::NodeId> {
            self.body().access(id, cx, nodes)
        }
    };
}
