pub use crate::*;

#[derive(Clone)]
struct TextEditorState {
    cursor: usize
}

pub fn text_editor(text: impl Binding<String>) -> impl View {
    state( TextEditorState{ cursor: 0 }, move |state| {
        let text = text.clone();
        canvas(move |rect, vger| {
            vger.translate([0.0, rect.height()]);
            let font_size = 18;
            let break_width = Some(rect.width());
            let s = text.get();
            vger.text(&s, font_size, TEXT_COLOR, break_width);
        }).key(move |k| {
            if k == KeyCode::ArrowLeft {
                state.with_mut(|s| s.cursor -= 1)
            }
            if k == KeyCode::ArrowRight {
                state.with_mut(|s| s.cursor += 1)
            }
        })
    })
}