use crate::*;

/// Struct for `circle`.
pub struct Circle {
    paint: Paint,
}

impl Circle {
    fn geom(&self, id: ViewId, cx: &mut Context) -> (LocalPoint, f32) {
        let rect = cx.layout.entry(id).or_default().rect;

        (rect.center(), rect.size.width.min(rect.size.height) / 2.0)
    }

    pub fn color(self, color: Color) -> Circle {
        Circle {
            paint: Paint::Color(color),
        }
    }
}

impl View for Circle {
    fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut Vger) {
        let (center, radius) = self.geom(id, cx);

        let paint = self.paint.vger_paint(vger);
        vger.fill_circle(center, radius, paint);
    }

    fn layout(&self, id: ViewId, sz: LocalSize, cx: &mut Context, _vger: &mut Vger) -> LocalSize {
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
        _vger: &mut Vger,
    ) -> Option<ViewId> {
        let (center, radius) = self.geom(id, cx);

        if pt.distance_to(center) < radius {
            Some(id)
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
pub struct Rectangle {
    corner_radius: f32,
    paint: Paint,
}

impl Rectangle {
    fn geom(&self, id: ViewId, cx: &mut Context) -> LocalRect {
        cx.layout.entry(id).or_default().rect
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
    fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut Vger) {
        let rect = self.geom(id, cx);

        let paint = self.paint.vger_paint(vger);
        vger.fill_rect(rect, self.corner_radius, paint);
    }

    fn layout(&self, id: ViewId, sz: LocalSize, cx: &mut Context, _vger: &mut Vger) -> LocalSize {
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
        _vger: &mut Vger,
    ) -> Option<ViewId> {
        let rect = self.geom(id, cx);

        if rect.contains(pt) {
            Some(id)
        } else {
            None
        }
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
