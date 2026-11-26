use crate::*;

/// Struct for `canvas`
#[derive(Clone)]
pub struct Canvas<F> {
    func: F,
}

impl<F> DynView for Canvas<F>
where
    F: Fn(&mut Context, LocalRect, &mut Vger) + Clone + 'static,
{
    fn draw(&self, vid: ViewId, args: &mut DrawArgs) {
        let rect = args.cx.get_layout(vid).rect;

        args.vger.save();
        (self.func)(args.cx, rect, args.vger);
        args.vger.restore();
    }

    fn layout(&self, vid: ViewId, args: &mut LayoutArgs) -> LocalSize {
        args.cx.update_layout(
            vid,
            LayoutBox {
                rect: LocalRect::new(LocalPoint::zero(), args.sz),
                offset: LocalOffset::zero(),
            },
        );
        args.sz
    }

    fn hittest(&self, vid: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let rect = cx.get_layout(vid).rect;

        if rect.contains(pt) { Some(vid) } else { None }
    }
}

/// Canvas for GPU drawing with Vger. See https://github.com/audulus/vger-rs.
pub fn canvas<F: Fn(&mut Context, LocalRect, &mut Vger) + Clone + 'static>(f: F) -> Canvas<F> {
    Canvas { func: f }
}

impl<F> private::Sealed for Canvas<F> {}
