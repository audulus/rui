use crate::*;

/// Struct for `circle`.
#[derive(Clone)]
pub struct Circle {
    paint: Paint,
}

impl Circle {
    fn geom(&self, vid: ViewId, cx: &mut Context) -> (LocalPoint, f32) {
        let rect = cx.get_layout(vid).rect;

        (rect.center(), rect.size.width.min(rect.size.height) / 2.0)
    }

    pub fn color(self, color: Color) -> Circle {
        Circle {
            paint: Paint::Color(color),
        }
    }
}

impl DynView for Circle {
    fn draw(&self, vid: ViewId, args: &mut DrawArgs) {
        let (center, radius) = self.geom(vid, args.cx);

        let vger = &mut args.vger;
        let paint = self.paint.vger_paint(vger);
        vger.fill_circle(center, radius, paint);
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
        let (center, radius) = self.geom(vid, cx);

        if pt.distance_to(center) < radius {
            Some(vid)
        } else {
            None
        }
    }
}

impl private::Sealed for Circle {}

/// Renders a circle which expands to fill available space.
pub fn circle() -> Circle {
    Circle {
        paint: Paint::Color(Color::CYAN),
    }
}

/// Struct for `rectangle`.
#[derive(Clone)]
pub struct Rectangle {
    corner_radius: f32,
    paint: Paint,
}

impl Rectangle {
    fn geom(&self, vid: ViewId, cx: &mut Context) -> LocalRect {
        cx.get_layout(vid).rect
    }

    /// Sets the fill color for the rectangle.
    pub fn color(self, color: Color) -> Rectangle {
        Rectangle {
            corner_radius: self.corner_radius,
            paint: Paint::Color(color),
        }
    }

    /// Sets the rectangle's corner radius.
    pub fn corner_radius(self, radius: f32) -> Rectangle {
        Rectangle {
            corner_radius: radius,
            paint: self.paint,
        }
    }
}

impl DynView for Rectangle {
    fn draw(&self, vid: ViewId, args: &mut DrawArgs) {
        let rect = self.geom(vid, args.cx);

        let vger = &mut args.vger;
        let paint = self.paint.vger_paint(vger);
        vger.fill_rect(rect, self.corner_radius, paint);
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
        let rect = self.geom(vid, cx);

        if rect.contains(pt) { Some(vid) } else { None }
    }
}

impl private::Sealed for Rectangle {}

/// Renders a rectangle which expands to fill available space.
pub fn rectangle() -> Rectangle {
    Rectangle {
        corner_radius: 0.0,
        paint: Paint::Color(Color::CYAN),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_layout_fills_available() {
        let mut cx = Context::new();
        let ui = circle();
        let sz = [80.0, 60.0].into();
        let vid = cx.root_id();
        let result = ui.layout(
            vid,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );
        assert_eq!(result, sz);
    }

    #[test]
    fn test_circle_hittest_inside() {
        let mut cx = Context::new();
        let ui = circle();
        let sz = [100.0, 100.0].into();
        let vid = cx.root_id();
        ui.layout(
            vid,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );
        // Center of a 100x100 circle is (50,50), radius 50
        assert!(ui.hittest(vid, [50.0, 50.0].into(), &mut cx).is_some());
    }

    #[test]
    fn test_circle_hittest_outside() {
        let mut cx = Context::new();
        let ui = circle();
        let sz = [100.0, 100.0].into();
        let vid = cx.root_id();
        ui.layout(
            vid,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );
        // Corner of square is outside circle
        assert!(ui.hittest(vid, [0.0, 0.0].into(), &mut cx).is_none());
    }

    #[test]
    fn test_rectangle_layout_fills_available() {
        let mut cx = Context::new();
        let ui = rectangle();
        let sz = [120.0, 40.0].into();
        let vid = cx.root_id();
        let result = ui.layout(
            vid,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );
        assert_eq!(result, sz);
    }

    #[test]
    fn test_rectangle_hittest_inside() {
        let mut cx = Context::new();
        let ui = rectangle();
        let sz = [100.0, 50.0].into();
        let vid = cx.root_id();
        ui.layout(
            vid,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );
        assert!(ui.hittest(vid, [50.0, 25.0].into(), &mut cx).is_some());
    }

    #[test]
    fn test_rectangle_hittest_outside() {
        let mut cx = Context::new();
        let ui = rectangle();
        let sz = [100.0, 50.0].into();
        let vid = cx.root_id();
        ui.layout(
            vid,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );
        assert!(ui.hittest(vid, [150.0, 25.0].into(), &mut cx).is_none());
    }
}
