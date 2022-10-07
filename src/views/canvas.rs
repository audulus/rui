use crate::*;

/// Struct for `canvas`
pub struct Canvas<F> {
    func: F,
}

impl<F> View for Canvas<F>
where
    F: Fn(&mut Context, LocalRect, &mut Vger) + 'static,
{
    fn draw(&self, id: ViewId, args: &mut DrawArgs) {
        let rect = args.cx.layout.entry(id).or_default().rect;

        args.vger.save();
        (self.func)(args.cx, rect, args.vger);
        args.vger.restore();
    }

    fn layout(&self, id: ViewId, args: &mut LayoutArgs) -> LocalSize {
        args.cx.layout.insert(
            id,
            LayoutBox {
                rect: LocalRect::new(LocalPoint::zero(), args.sz),
                offset: LocalOffset::zero(),
            },
        );
        args.sz
    }

    fn hittest(
        &self,
        id: ViewId,
        pt: LocalPoint,
        cx: &mut Context,
        _vger: &mut Vger,
    ) -> Option<ViewId> {
        let rect = cx.layout.entry(id).or_default().rect;

        if rect.contains(pt) {
            Some(id)
        } else {
            None
        }
    }
}

/// Canvas for GPU drawing with Vger. See https://github.com/audulus/vger-rs.
pub fn canvas<F: Fn(&mut Context, LocalRect, &mut Vger) + 'static>(f: F) -> impl View {
    Canvas { func: f }
}

impl<F> private::Sealed for Canvas<F> {}
