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

    fn find_line(&self) -> usize {
        let mut i = 0;
        for line in &self.glyph_info.borrow().lines {
            if self.cursor >= line.glyph_start && self.cursor < line.glyph_end {
                break;
            }
            i += 1;
        }
        i
    }

    fn closest_in_range(&self, p: LocalPoint, range: std::ops::Range<usize>, rects: &Vec<LocalRect>) -> usize {
        let mut d = std::f32::MAX;
        let mut closest = 0;
        for i in range {
            let dp = rects[i].center().distance_to(p);
            if dp < d {
                closest = i;
                d = dp;
            }
        }
        closest
    }

    fn down(&mut self) {
        let info = self.glyph_info.borrow();
        let rects = &info.glyph_rects;
        let p = rects[self.cursor].center();
        
        let line = self.find_line() + 1;
        if line < self.glyph_info.borrow().lines.len() {
            let metrics = self.glyph_info.borrow().lines[line];
            self.cursor = self.closest_in_range(p, metrics.glyph_start..metrics.glyph_end, rects);
        }
        
    }

    fn up(&mut self) {
        let info = self.glyph_info.borrow();
        let rects = &info.glyph_rects;
        let p = rects[self.cursor].center();
        
        let line = self.find_line();
        if line > 0 {
            let metrics = self.glyph_info.borrow().lines[line-1];
            self.cursor = self.closest_in_range(p, metrics.glyph_start..metrics.glyph_end, rects);
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
                KeyPress::ArrowUp => state.with_mut(|s| s.up() ),
                KeyPress::ArrowDown => state.with_mut(|s| s.down() ),
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
                KeyPress::Home => {
                    state.with_mut(|s| s.cursor = 0 )
                },
                KeyPress::End => {
                    state.with_mut(|s| s.cursor = len-1 )
                }
                _ => ()
            }
        })
    })
}