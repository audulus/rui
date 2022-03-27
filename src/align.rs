use crate::*;

pub enum HAlignment {
    Left,
    Center,
    Right,
}

pub fn align_h(child: LocalRect, parent: LocalRect, align: HAlignment) -> LocalOffset {
    let c_off = parent.center() - child.center();
    match align {
        HAlignment::Left => [parent.min_x() - child.min_x(), c_off.y].into(),
        HAlignment::Center => c_off,
        HAlignment::Right => [parent.max_x() - child.max_x(), c_off.y].into(),
    }
}

pub enum VAlignment {
    Top,
    Middle,
    Bottom,
}

pub fn align_v(child: LocalRect, parent: LocalRect, align: VAlignment) -> LocalOffset {
    let c_off = parent.center() - child.center();
    match align {
        VAlignment::Top => [c_off.x, parent.max_y() - child.max_y()].into(),
        VAlignment::Middle => c_off,
        VAlignment::Bottom => [c_off.x, parent.min_y() - child.min_y()].into(),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn rect<A, B>(origin: A, size: B) -> LocalRect where A: Into<LocalPoint>, B: Into<LocalSize> {
        LocalRect::new(origin.into(), size.into())
    }

    #[test]
    fn test_align_h() {

        let parent = rect([0.0,0.0], [10.0,10.0]);

        let off = align_h(rect([0.0,0.0], [1.0,1.0]), parent, HAlignment::Center);
        assert_eq!(off.x, 4.5);
        assert_eq!(off.y, 4.5);

        let off = align_h(rect([0.0,0.0], [1.0,1.0]), parent, HAlignment::Left);
        assert_eq!(off.x, 0.0);
        assert_eq!(off.y, 4.5);

        let off = align_h(rect([0.0,0.0], [1.0,1.0]), parent, HAlignment::Right);
        assert_eq!(off.x, 9.0);
        assert_eq!(off.y, 4.5);

    }

    #[test]
    fn test_align_v() {

        let parent = rect([0.0,0.0], [10.0,10.0]);

        let off = align_v(rect([0.0,0.0], [1.0,1.0]), parent, VAlignment::Middle);
        assert_eq!(off.x, 4.5);
        assert_eq!(off.y, 4.5);

        let off = align_v(rect([0.0,0.0], [1.0,1.0]), parent, VAlignment::Bottom);
        assert_eq!(off.x, 4.5);
        assert_eq!(off.y, 0.0);

        let off = align_v(rect([0.0,0.0], [1.0,1.0]), parent, VAlignment::Top);
        assert_eq!(off.x, 4.5);
        assert_eq!(off.y, 9.0);

    }

}