use crate::*;

/// Struct for `circle`.
#[derive(Clone)]
pub struct Circle {
    paint: Paint,
}

impl Circle {
    fn geom(&self, path: &IdPath, cx: &mut Context) -> (LocalPoint, f32) {
        let rect = cx.layout.get(path).map(|b| b.rect).unwrap_or_default();

        (rect.center(), rect.size.width.min(rect.size.height) / 2.0)
    }

    pub fn color(self, color: Color) -> Circle {
        Circle {
            paint: Paint::Color(color),
        }
    }
}

impl View for Circle {
    fn draw(&self, path: &mut IdPath, args: &mut DrawArgs) {
        let (center, radius) = self.geom(path, args.cx);

        let vger = &mut args.vger;
        let paint = self.paint.vger_paint(vger);
        vger.fill_circle(center, radius, paint);
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
        let (center, radius) = self.geom(path, cx);

        if pt.distance_to(center) < radius {
            Some(hash(path))
        } else {
            None
        }
    }

    fn gc(&self, path: &mut IdPath, _cx: &mut Context, map: &mut Vec<ViewId>) {
        map.push(hash(path));
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
        cx.layout.get(path).map(|b| b.rect).unwrap_or_default()
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

impl View for Rectangle {
    fn draw(&self, path: &mut IdPath, args: &mut DrawArgs) {
        let rect = self.geom(path, args.cx);

        let vger = &mut args.vger;
        let paint = self.paint.vger_paint(vger);
        vger.fill_rect(rect, self.corner_radius, paint);
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
        let rect = self.geom(path, cx);

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

impl private::Sealed for Rectangle {}

/// Renders a rectangle which expands to fill available space.
pub fn rectangle() -> Rectangle {
    Rectangle {
        corner_radius: 0.0,
        paint: Paint::Color(Color::CYAN),
    }
}
