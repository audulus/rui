pub use crate::*;

#[derive(Clone)]
struct TextEditorState {
    cursor: usize
}

impl TextEditorState {
    fn fwd(&mut self, len: usize) {
        self.cursor += 1;
        if self.cursor >= len {
            self.cursor = len-1;
        }
    }
    fn back(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }
}

pub fn text_editor(text: impl Binding<String>) -> impl View {
    let len = text.get().len();
    state( TextEditorState{ cursor: 0 }, move |state| {
        let text = text.clone();
        canvas(move |rect, vger| {
            vger.translate([0.0, rect.height()]);
            let font_size = 18;
            let break_width = Some(rect.width());
            vger.text(&text.get(), font_size, TEXT_COLOR, break_width);
        }).key(move |k| {
            match k {
                KeyCode::ArrowLeft => state.with_mut(|s| s.fwd(len) ),
                KeyCode::ArrowRight => state.with_mut(|s| s.back() ),
                _ => ()
            }
        })
    })
}