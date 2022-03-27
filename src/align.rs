use crate::*;

pub enum HAlignment {
    Leading,
    Center,
    Trailing,
}

pub fn align_h(child: LocalRect, parent: LocalRect, align: HAlignment) -> LocalOffset {
    let c_off = parent.center() - child.center();
    match align {
        HAlignment::Leading => [parent.min_x() - child.min_x(), c_off.y].into(),
        HAlignment::Center => c_off,
        HAlignment::Trailing => [parent.max_x() - child.max_x(), c_off.y].into(),
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

pub fn align(child: LocalRect, parent: LocalRect, halign: HAlignment, valign: VAlignment) -> LocalOffset {
    let c_off = parent.center() - child.center();
    LocalOffset::new(
        match halign {
            HAlignment::Leading => parent.min_x() - child.min_x(),
            HAlignment::Center => c_off.x,
            HAlignment::Trailing => parent.max_x() - child.max_x()
        },
        match valign {
            VAlignment::Top => parent.max_y() - child.max_y(),
            VAlignment::Middle => c_off.y,
            VAlignment::Bottom => parent.min_y() - child.min_y(),
        }
    )
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

        let off = align_h(rect([0.0,0.0], [1.0,1.0]), parent, HAlignment::Leading);
        assert_eq!(off.x, 0.0);
        assert_eq!(off.y, 4.5);

        let off = align_h(rect([0.0,0.0], [1.0,1.0]), parent, HAlignment::Trailing);
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

    #[test]
    fn test_align() {
        let parent = rect([0.0,0.0], [10.0,10.0]);

        let off = align(rect([0.0,0.0], [1.0,1.0]), parent, HAlignment::Center, VAlignment::Middle);
        assert_eq!(off.x, 4.5);
        assert_eq!(off.y, 4.5);

        let off = align(rect([0.0,0.0], [1.0,1.0]), parent, HAlignment::Leading, VAlignment::Bottom);
        assert_eq!(off.x, 0.0);
        assert_eq!(off.y, 0.0);
    }

}