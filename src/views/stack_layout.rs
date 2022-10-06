
pub enum StackItem {
    Fixed(f32),
    Flexible,
}

/// 1-D stack layout to make the algorithm clear.
///
/// Returns length used to express the layout. If there are any
/// flexible items, will return `total`, since the flexible items
/// will expand to fill the available space.
pub fn stack_layout(
    total: f32,
    sizes: &[StackItem],
    intervals: &mut [(f32, f32)],
    flex_length: &mut f32,
) -> f32 {
    assert_eq!(sizes.len(), intervals.len());

    // Count the number of flexible items and total of fixed sizes.
    let mut flex_count = 0;
    let mut sizes_sum = 0.0;
    for sz in sizes {
        match sz {
            StackItem::Flexible => flex_count += 1,
            StackItem::Fixed(s) => sizes_sum += s,
        }
    }

    // length of flexible items is remaining size divided equally
    *flex_length = (total - sizes_sum) / (flex_count as f32);

    let mut x = 0.0;
    for i in 0..sizes.len() {
        let sz = match sizes[i] {
            StackItem::Flexible => *flex_length,
            StackItem::Fixed(s) => s,
        };

        intervals[i] = (x, x + sz);
        x += sz;
    }

    x
}