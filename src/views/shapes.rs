use crate::*;

/// Struct for `circle`.
#[derive(Clone)]
pub struct Circle {
    paint: Paint,
}

impl Circle {
    fn geom(&self, path: &IdPath, cx: &mut Context) -> (LocalPoint, f32) {
        let rect = cx.get_layout(path).rect;

        (rect.center(), rect.size.width.min(rect.size.height) / 2.0)
    }

    pub fn color(self, color: Color) -> Circle {
        Circle {
            paint: Paint::Color(color),
        }
    }
}

impl DynView for Circle {
    fn draw(&self, path: &mut IdPath, args: &mut DrawArgs) {
        let (center, radius) = self.geom(path, args.cx);

        let vger = &mut args.vger;
        let paint = self.paint.vger_paint(vger);
        vger.fill_circle(center, radius, paint);
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        args.cx.update_layout(
            path,
            LayoutBox {
                rect: LocalRect::new(LocalPoint::zero(), args.sz),
                offset: LocalOffset::zero(),
            },
        );
        args.sz
    }

    fn hittest(&self, path: &mut IdPath, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let (center, radius) = self.geom(path, cx);

        if pt.distance_to(center) < radius {
            Some(cx.view_id(path))
        } else {
            None
        }
    }

    fn gc(&self, path: &mut IdPath, cx: &mut Context, map: &mut Vec<ViewId>) {
        map.push(cx.view_id(path));
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
    fn geom(&self, path: &IdPath, cx: &mut Context) -> LocalRect {
        cx.get_layout(path).rect
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
    fn draw(&self, path: &mut IdPath, args: &mut DrawArgs) {
        let rect = self.geom(path, args.cx);

        let vger = &mut args.vger;
        let paint = self.paint.vger_paint(vger);
        vger.fill_rect(rect, self.corner_radius, paint);
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        args.cx.update_layout(
            path,
            LayoutBox {
                rect: LocalRect::new(LocalPoint::zero(), args.sz),
                offset: LocalOffset::zero(),
            },
        );
        args.sz
    }

    fn hittest(&self, path: &mut IdPath, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let rect = self.geom(path, cx);

        if rect.contains(pt) {
            Some(cx.view_id(path))
        } else {
            None
        }
    }

    fn gc(&self, path: &mut IdPath, cx: &mut Context, map: &mut Vec<ViewId>) {
        map.push(cx.view_id(path));
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
        let mut path = vec![0];
        let result = ui.layout(
            &mut path,
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
        let mut path = vec![0];
        ui.layout(
            &mut path,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );
        // Center of a 100x100 circle is (50,50), radius 50
        assert!(ui.hittest(&mut path, [50.0, 50.0].into(), &mut cx).is_some());
    }

    #[test]
    fn test_circle_hittest_outside() {
        let mut cx = Context::new();
        let ui = circle();
        let sz = [100.0, 100.0].into();
        let mut path = vec![0];
        ui.layout(
            &mut path,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );
        // Corner of square is outside circle
        assert!(ui.hittest(&mut path, [0.0, 0.0].into(), &mut cx).is_none());
    }

    #[test]
    fn test_rectangle_layout_fills_available() {
        let mut cx = Context::new();
        let ui = rectangle();
        let sz = [120.0, 40.0].into();
        let mut path = vec![0];
        let result = ui.layout(
            &mut path,
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
        let mut path = vec![0];
        ui.layout(
            &mut path,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );
        assert!(ui.hittest(&mut path, [50.0, 25.0].into(), &mut cx).is_some());
    }

    #[test]
    fn test_rectangle_hittest_outside() {
        let mut cx = Context::new();
        let ui = rectangle();
        let sz = [100.0, 50.0].into();
        let mut path = vec![0];
        ui.layout(
            &mut path,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );
        assert!(ui.hittest(&mut path, [150.0, 25.0].into(), &mut cx).is_none());
    }
}
