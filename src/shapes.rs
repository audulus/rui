use crate::*;

pub struct Circle {}

impl Circle {
    fn geom(&self, id: ViewID, cx: &mut Context) -> (LocalPoint, f32) {
        let rect = cx
                .layout
                .entry(id)
                .or_insert(LayoutBox::default())
                .rect;

        (rect.center(), rect.size.width.min(rect.size.height) / 2.0)
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

        let paint = vger.color_paint(Color::CYAN);
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

    fn hittest(&self, id: ViewID, pt: LocalPoint, cx: &mut Context, vger: &mut VGER) -> Option<ViewID> {
        let (center, radius) = self.geom(id, cx);

        if pt.distance_to(center) < radius { Some(id) } else { None }
    }
}

pub fn circle() -> Circle {
    Circle {}
}

pub struct Rectangle {
    corner_radius: f32
}

pub fn rectangle(corner_radius: f32) -> Rectangle {
    Rectangle {
        corner_radius
    }
}
