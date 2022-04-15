use euclid::*;

/// Region type cribbed from Druid.
#[derive(Clone, Debug)]
pub struct Region<Space> {
    rects: Vec<Rect<f32, Space>>,
}

impl<Space> Region<Space> {
    /// The empty region.
    pub const EMPTY: Region<Space> = Region { rects: Vec::new() };

    /// Returns the collection of rectangles making up this region.
    #[inline]
    pub fn rects(&self) -> &[Rect<f32, Space>] {
        &self.rects
    }

    /// Adds a rectangle to this region.
    pub fn add_rect(&mut self, rect: Rect<f32, Space>) {
        if !rect.is_empty() {
            self.rects.push(rect);
        }
    }

    /// Replaces this region with a single rectangle.
    pub fn set_rect(&mut self, rect: Rect<f32, Space>) {
        self.clear();
        self.add_rect(rect);
    }

    /// Sets this region to the empty region.
    pub fn clear(&mut self) {
        self.rects.clear();
    }

    /// Returns a rectangle containing this region.
    pub fn bounding_box(&self) -> Rect<f32, Space> {
        if self.rects.is_empty() {
            Rect::<f32, Space>::default()
        } else {
            self.rects[1..]
                .iter()
                .fold(self.rects[0], |r, s| r.union(s))
        }
    }

    /// Returns `true` if this region has a non-empty intersection with the given rectangle.
    pub fn intersects(&self, rect: Rect<f32, Space>) -> bool {
        self.rects.iter().any(|r| r.intersects(&rect))
    }

    /// Returns `true` if this region is empty.
    pub fn is_empty(&self) -> bool {
        // Note that we only ever add non-empty rects to self.rects.
        self.rects.is_empty()
    }

    /// Modifies this region by including everything in the other region.
    pub fn union_with(&mut self, other: &Region<Space>) {
        self.rects.extend_from_slice(&other.rects);
    }

    // /// Modifies this region by intersecting it with the given rectangle.
    // pub fn intersect_with(&mut self, rect: WorldRect) {
    //     // TODO: this would be a good use of the nightly drain_filter function, if it stabilizes
    //     for r in &mut self.rects {
    //         *r = r.intersect(rect);
    //     }
    //     self.rects.retain(|r| r.area() > 0.0)
    // }
}

impl<Space> std::ops::AddAssign<Vector2D<f32, Space>> for Region<Space> {
    fn add_assign(&mut self, rhs: Vector2D<f32, Space>) {
        for r in &mut self.rects {
            *r = r.translate(rhs)
        }
    }
}

impl<Space> std::ops::SubAssign<Vector2D<f32, Space>> for Region<Space> {
    fn sub_assign(&mut self, rhs: Vector2D<f32, Space>) {
        for r in &mut self.rects {
            *r = r.translate(-rhs)
        }
    }
}

impl<Space> From<Rect<f32, Space>> for Region<Space> {
    fn from(rect: Rect<f32, Space>) -> Region<Space> {
        Region { rects: vec![rect] }
    }
}
