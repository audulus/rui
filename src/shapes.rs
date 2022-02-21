use crate::*;

pub struct Circle {
    paint: Paint,
}

impl Circle {
    fn geom(&self, id: ViewID, cx: &mut Context) -> (LocalPoint, f32) {
        let rect = cx.layout.entry(id).or_insert(LayoutBox::default()).rect;

        (rect.center(), rect.size.width.min(rect.size.height) / 2.0)
    }

    pub fn color(self, color: Color) -> Circle {
        Circle {
            paint: Paint::Color(color),
        }
    }
}

impl View for Circle {
    fn print(&self, _id: ViewID, _cx: &mut Context) {
        println!("circle");
    }

    fn process(&self, _event: &Event, _id: ViewID, _cx: &mut Context) {
        // do nothing
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        let (center, radius) = self.geom(id, cx);

        let paint = self.paint.vger_paint(vger);
        vger.fill_circle(center, radius, paint);
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
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
        id: ViewID,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewID> {
        let (center, radius) = self.geom(id, cx);

        if pt.distance_to(center) < radius {
            Some(id)
        } else {
            None
        }
    }
}

pub fn circle() -> Circle {
    Circle {
        paint: Paint::Color(Color::CYAN),
    }
}

pub struct Rectangle {
    corner_radius: f32,
    paint: Paint,
}

impl Rectangle {
    fn geom(&self, id: ViewID, cx: &mut Context) -> LocalRect {
        cx.layout.entry(id).or_insert(LayoutBox::default()).rect
    }

    pub fn color(self, color: Color) -> Rectangle {
        Rectangle {
            corner_radius: self.corner_radius,
            paint: Paint::Color(color),
        }
    }
}

impl View for Rectangle {
    fn print(&self, _id: ViewID, _cx: &mut Context) {
        println!("rectangle");
    }

    fn process(&self, _event: &Event, _id: ViewID, _cx: &mut Context) {
        // do nothing
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        let rect = self.geom(id, cx);

        let paint = self.paint.vger_paint(vger);
        vger.fill_rect(
            rect.origin,
            rect.origin + rect.size,
            self.corner_radius,
            paint,
        );
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, _vger: &mut VGER) -> LocalSize {
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
        id: ViewID,
        pt: LocalPoint,
        cx: &mut Context,
        _vger: &mut VGER,
    ) -> Option<ViewID> {
        let rect = self.geom(id, cx);

        if rect.contains(pt) {
            Some(id)
        } else {
            None
        }
    }
}

pub fn rectangle(corner_radius: f32) -> Rectangle {
    Rectangle {
        corner_radius,
        paint: Paint::Color(Color::CYAN),
    }
}
