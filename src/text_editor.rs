pub use crate::*;
use std::cell::RefCell;
use std::rc::Rc;


struct TextEditorGlyphInfo {
    glyph_rects: Vec<LocalRect>,
    lines: Vec<LineMetrics>,
}

impl TextEditorGlyphInfo {
    fn new() -> Self {
        Self { glyph_rects: vec![], lines: vec![] }
    }
}

#[derive(Clone)]
struct TextEditorState {
    cursor: usize,
    glyph_info: Rc<RefCell<TextEditorGlyphInfo>>
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

impl TextEditorState {
    fn new() -> Self {
        Self {
            cursor: 0,
            glyph_info: Rc::new(RefCell::new(TextEditorGlyphInfo::new()))
        }
    }
}

pub fn text_editor(text: impl Binding<String>) -> impl View {
    let len = text.get().len();
    state( TextEditorState::new(), move |state| {
        let text = text.clone();
        let text2 = text.clone();
        let cursor = state.with(|s| s.cursor);
        let state2 = state.clone();
        canvas(move |rect, vger| {
            vger.translate([0.0, rect.height()]);
            let font_size = 18;
            let break_width = Some(rect.width());

            let rects = vger.glyph_positions(&text.get(), font_size, break_width);
            let glyph_rect_paint = vger.color_paint(vger::Color::MAGENTA);
            vger.fill_rect(rects[cursor], 0.0, glyph_rect_paint);

            let lines = vger.line_metrics(&text.get(), font_size, break_width);
            state2.get().glyph_info.borrow_mut().glyph_rects = rects;
            state2.get().glyph_info.borrow_mut().lines = lines;

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
                KeyPress::Character(c) => {
                    text2.with_mut(|t| { 
                        t.insert_str(cursor, c);
                        state.with_mut(|s| s.cursor += c.len())
                    });
                },
                KeyPress::Space => {
                    text2.with_mut(|t| { 
                        t.insert_str(cursor, " ");
                        state.with_mut(|s| s.cursor += 1)
                    });
                },
                _ => ()
            }
        })
    })
}