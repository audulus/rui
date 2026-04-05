use crate::*;

#[derive(Clone)]
pub struct Spacer {}

impl DynView for Spacer {
    fn draw(&self, _path: &mut IdPath, _args: &mut DrawArgs) {}
    fn layout(&self, _path: &mut IdPath, _args: &mut LayoutArgs) -> LocalSize {
        [0.0, 0.0].into()
    }

    fn is_flexible(&self) -> bool {
        true
    }
}

impl private::Sealed for Spacer {}

/// Inserts a flexible space in a stack.
pub fn spacer() -> Spacer {
    Spacer {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spacer_zero_size() {
        let mut cx = Context::new();
        let ui = spacer();
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

    #[test]
    fn test_spacer_is_flexible() {
        let ui = spacer();
        assert!(ui.is_flexible());
    }
}
