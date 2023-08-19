use crate::*;

/// Struct for `canvas`
#[derive(Clone)]
pub struct Canvas<F> {
    func: F,
}

impl<F> View for Canvas<F>
where
    F: Fn(&mut Context, LocalRect, &mut Vger) + 'static,
{
    fn draw(&self, path: &mut IdPath, args: &mut DrawArgs) {
        let rect = args.cx.layout.entry(path.clone()).or_default().rect;

        args.vger.save();
        (self.func)(args.cx, rect, args.vger);
        args.vger.restore();
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        args.cx.layout.insert(
            path.clone(),
            LayoutBox {
                rect: LocalRect::new(LocalPoint::zero(), args.sz),
                offset: LocalOffset::zero(),
            },
        );
        args.sz
    }

    fn hittest(&self, path: &mut IdPath, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let rect = cx.layout.entry(path.clone()).or_default().rect;

        if rect.contains(pt) {
            Some(hash(path))
        } else {
            None
        }
    }

    fn gc(&self, path: &mut IdPath, _cx: &mut Context, map: &mut Vec<ViewId>) {
        map.push(hash(path));
    }
}

/// Canvas for GPU drawing with Vger. See https://github.com/audulus/vger-rs.
pub fn canvas<F: Fn(&mut Context, LocalRect, &mut Vger) + 'static>(f: F) -> impl View {
    Canvas { func: f }
}

impl<F> private::Sealed for Canvas<F> {}
