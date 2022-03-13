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
        let text2 = text.clone();
        let cursor = state.with(|s| s.cursor);
        canvas(move |rect, vger| {
            vger.translate([0.0, rect.height()]);
            let font_size = 18;
            let break_width = Some(rect.width());

            let rects = vger.glyph_positions(&text.get(), font_size, break_width);
            let glyph_rect_paint = vger.color_paint(vger::Color::MAGENTA);
            vger.fill_rect(rects[cursor], 0.0, glyph_rect_paint);

            vger.text(&text.get(), font_size, TEXT_COLOR, break_width);
            
        }).key(move |k| {
            match k {
                KeyPress::ArrowLeft => state.with_mut(|s| s.back() ),
                KeyPress::ArrowRight => state.with_mut(|s| s.fwd(len) ),
                KeyPress::Backspace => {
                    if cursor > 0 {
                        text2.with_mut(|t| { 
                            t.remove(cursor-1);
                        });
                        state.with_mut(|s| s.back() );
                    }
                },
                _ => ()
            }
        })
    })
}