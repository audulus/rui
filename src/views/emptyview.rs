use crate::*;

#[derive(Clone)]
pub struct EmptyView {}

impl DynView for EmptyView {
    fn draw(&self, _path: &mut IdPath, _args: &mut DrawArgs) {}
    fn layout(&self, _path: &mut IdPath, _args: &mut LayoutArgs) -> LocalSize {
        [0.0, 0.0].into()
    }
}

impl private::Sealed for EmptyView {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emptyview_zero_size() {
        let mut cx = Context::new();
        let ui = EmptyView {};
        let sz = [100.0, 100.0].into();
        let mut path = vec![0];
        let result = ui.layout(
            &mut path,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );
        assert_eq!(result, [0.0, 0.0].into());
    }
}
