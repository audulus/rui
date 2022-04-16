use crate::*;

/// Struct for `canvas`
pub struct Canvas<F> {
    func: F,
}

impl<F> View for Canvas<F>
where
    F: Fn(&mut Context, LocalRect, &mut VGER) + 'static,
{
    fn print(&self, _id: ViewId, _cx: &mut Context) {
        println!("canvas");
    }

    fn process(&self, _event: &Event, _id: ViewId, _cx: &mut Context, _vger: &mut VGER) {
        // do nothing
    }

    fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut VGER) {
        let rect = cx.layout.entry(id).or_default().rect;

        vger.save();
        (self.func)(cx, rect, vger);
        vger.restore();
    }

    fn layout(&self, id: ViewId, sz: LocalSize, cx: &mut Context, _vger: &mut VGER) -> LocalSize {
        cx.layout.insert(
            id,
            LayoutBox {
                rect: LocalRect::new(LocalPoint::zero(), sz),
                offset: LocalOffset::zero(),
            },
        );
        sz
    }

    fn hittest(
        &self,
        id: ViewId,
        pt: LocalPoint,
        cx: &mut Context,
        _vger: &mut VGER,
    ) -> Option<ViewId> {
        let rect = cx.layout.entry(id).or_default().rect;

        if rect.contains(pt) {
            Some(id)
        } else {
            None
        }
    }
}

/// Canvas for GPU drawing with VGER. See https://github.com/audulus/vger-rs.
pub fn canvas<F: Fn(&mut Context, LocalRect, &mut VGER) + 'static>(f: F) -> impl View {
    Canvas { func: f }
}

impl<F> private::Sealed for Canvas<F> {}
