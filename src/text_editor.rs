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
            vger.text(&text.get(), font_size, TEXT_COLOR, break_width);
        }).key(move |k| {
            match k {
                KeyCode::ArrowLeft => state.with_mut(|s| s.cursor -= 1),
                KeyCode::ArrowRight => state.with_mut(|s| s.cursor += 1),
                _ => ()
            }
        })
    })
}