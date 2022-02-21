
pub struct Circle {}

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
