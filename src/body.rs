use crate::*;

// Using this depends on https://github.com/rust-lang/rust/issues/63063
pub trait Body: View {
    type V: View;
    fn body(&self) -> Self::V;

    fn print(&self, id: ViewID, cx: &mut Context) {
        self.body().print(id, cx)
    }

    fn process(&self, event: &Event, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        self.body().process(event, id, cx, vger)
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        self.body().draw(id, cx, vger)
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        self.body().layout(id, sz, cx, vger)
    }

    fn hittest(
        &self,
        id: ViewID,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewID> {
        self.body().hittest(id, pt, cx, vger)
    }

    fn commands(&self, id: ViewID, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        self.body().commands(id, cx, cmds);
    }
}

#[macro_export]
macro_rules! body_view {
    () => {
        fn print(&self, id: ViewID, cx: &mut Context) {
            self.body().print(id, cx)
        }

        fn process(&self, event: &Event, id: ViewID, cx: &mut Context, vger: &mut VGER) {
            self.body().process(event, id, cx, vger)
        }

        fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
            self.body().draw(id, cx, vger)
        }

        fn layout(
            &self,
            id: ViewID,
            sz: LocalSize,
            cx: &mut Context,
            vger: &mut VGER,
        ) -> LocalSize {
            self.body().layout(id, sz, cx, vger)
        }

        fn hittest(
            &self,
            id: ViewID,
            pt: LocalPoint,
            cx: &mut Context,
            vger: &mut VGER,
        ) -> Option<ViewID> {
            self.body().hittest(id, pt, cx, vger)
        }

        fn commands(&self, id: ViewID, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
            self.body().commands(id, cx, cmds);
        }
    };
}
