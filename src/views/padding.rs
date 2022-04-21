use crate::*;

/// Struct for the `padding` modifier.
pub struct Padding<V> {
    child: V,
    padding: f32,
}

impl<V> View for Padding<V>
where
    V: View,
{
    fn print(&self, id: ViewId, cx: &mut Context) {
        println!("Padding {{");
        (self.child).print(id.child(&0), cx);
        println!("}}");
    }

    fn process(&self, event: &Event, id: ViewId, cx: &mut Context, vger: &mut Vger) {
        let mut local_event = event.clone();
        local_event.position -= LocalOffset::new(self.padding, self.padding);
        self.child.process(&local_event, id.child(&0), cx, vger);
    }

    fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut Vger) {
        vger.save();
        vger.translate([self.padding, self.padding]);
        self.child.draw(id.child(&0), cx, vger);
        vger.restore();
    }

    fn layout(&self, id: ViewId, sz: LocalSize, cx: &mut Context, vger: &mut Vger) -> LocalSize {
        let child_size = self.child.layout(
            id.child(&0),
            sz - [2.0 * self.padding, 2.0 * self.padding].into(),
            cx,
            vger,
        );
        child_size + LocalSize::new(2.0 * self.padding, 2.0 * self.padding)
    }

    fn dirty(
        &self,
        id: ViewId,
        xform: LocalToWorld,
        cx: &mut Context,
    ) {
        self.child.dirty(
            id.child(&0),
            xform.pre_translate([self.padding, self.padding].into()),
            cx,
        );
    }

    fn hittest(
        &self,
        id: ViewId,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut Vger,
    ) -> Option<ViewId> {
        self.child.hittest(
            id.child(&0),
            pt - LocalOffset::new(self.padding, self.padding),
            cx,
            vger,
        )
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

pub enum PaddingParam {
    Auto,
    Px(f32),
}
pub struct Auto;
impl From<Auto> for PaddingParam {
    fn from(_val: Auto) -> Self {
        PaddingParam::Auto
    }
}
impl From<f32> for PaddingParam {
    fn from(val: f32) -> Self {
        PaddingParam::Px(val)
    }
}

impl<V> Padding<V>
where
    V: View,
{
    pub fn new(child: V, param: PaddingParam) -> Self {
        Self {
            child,
            padding: match param {
                PaddingParam::Auto => 5.0,
                PaddingParam::Px(px) => px,
            },
        }
    }
}

impl<V> private::Sealed for Padding<V> {}
