pub use crate::*;

pub struct Tap<V: View> {
    child: V,
    func: Box<dyn Fn()>,
}

impl<V> Tap<V>
where
    V: View,
{
    pub fn new<F: Fn() + 'static>(v: V, f: F) -> Self {
        Self {
            child: v,
            func: Box::new(f),
        }
    }
}

impl<V> View for Tap<V>
where
    V: View,
{
    fn print(&self, id: ViewID, cx: &mut Context) {
        println!("Tap {{");
        (self.child).print(id.child(0), cx);
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
                    (*self.func)();
                }
            }
            _ => (),
        }
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        self.child.draw(id.child(0), cx, vger)
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        self.child.layout(id.child(0), sz, cx, vger)
    }

    fn hittest(
        &self,
        id: ViewID,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewID> {
        self.child.hittest(id.child(0), pt, cx, vger)
    }
}

pub enum GestureState {
    Began,
    Changed,
    Ended
}

pub struct Drag<V: View> {
    child: V,
    func: Box<dyn Fn(LocalOffset, GestureState)>,
}

impl<V> Drag<V>
where
    V: View,
{
    pub fn new<F: Fn(LocalOffset, GestureState) + 'static>(v: V, f: F) -> Self {
        Self {
            child: v,
            func: Box::new(f),
        }
    }
}

impl<V> View for Drag<V>
where
    V: View,
{
    fn print(&self, id: ViewID, cx: &mut Context) {
        println!("Drag {{");
        (self.child).print(id.child(0), cx);
        println!("}}");
    }

    fn process(&self, event: &Event, vid: ViewID, cx: &mut Context, vger: &mut VGER) {
        match &event.kind {
            EventKind::TouchBegin { id } => {
                if let Some(_) = self.hittest(vid, event.position, cx, vger) {
                    cx.touches[*id] = vid;
                    cx.starts[*id] = event.position;
                }
            }
            EventKind::TouchMove { id } => {
                if cx.touches[*id] == vid {
                    (*self.func)(event.position - cx.starts[*id], GestureState::Changed);
                }
            }
            EventKind::TouchEnd { id } => {
                if cx.touches[*id] == vid {
                    cx.touches[*id] = ViewID::default();
                    (*self.func)(event.position - cx.starts[*id], GestureState::Ended);
                }
            }
            _ => (),
        }
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        self.child.draw(id.child(0), cx, vger)
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        self.child.layout(id.child(0), sz, cx, vger)
    }

    fn hittest(
        &self,
        id: ViewID,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewID> {
        self.child.hittest(id.child(0), pt, cx, vger)
    }
}
