use crate::*;

/// View-model for `text_editor`.
struct TextEditorState {
    cursor: usize,
    glyph_rects: Vec<LocalRect>,
    lines: Vec<LineMetrics>,
}

impl TextEditorState {
    /// Returns the position of the cursor in local coordinates.
    fn cursor_pos(&self) -> LocalPoint {
        if self.cursor == self.glyph_rects.len() {
            if let Some(r) = self.glyph_rects.last() {
                [r.origin.x + r.size.width, r.origin.y].into()
            } else {
                [0.0, -20.0].into()
            }
        } else {
            self.glyph_rects[self.cursor].origin
        }
    }

    fn fwd(&mut self, len: usize) {
        self.cursor += 1;
        if self.cursor > len {
            self.cursor = len;
        }
    }
    fn back(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    fn find_line(&self) -> usize {
        let mut i = 0;
        for line in &self.lines {
            if self.cursor >= line.glyph_start && self.cursor < line.glyph_end {
                break;
            }
            i += 1;
        }
        // Ensure we don't go out of bounds.
        i = i.min(self.lines.len() - 1);
        i
    }

    fn closest_in_range(
        &self,
        p: LocalPoint,
        range: std::ops::Range<usize>,
        rects: &[LocalRect],
    ) -> usize {
        let mut d = std::f32::MAX;
        let mut closest = 0;
        for i in range {
            let dp = rects[i].origin.distance_to(p);
            if dp < d {
                closest = i;
                d = dp;
            }
        }
        closest
    }

    fn down(&mut self) {
        let p = self.cursor_pos();

        let line = self.find_line() + 1;
        if line < self.lines.len() {
            let metrics = self.lines[line];
            self.cursor =
                self.closest_in_range(p, metrics.glyph_start..metrics.glyph_end, &self.glyph_rects);
        }
    }

    fn up(&mut self) {
        let p = self.cursor_pos();

        let line = self.find_line();
        if line > 0 {
            let metrics = self.lines[line - 1];
            self.cursor =
                self.closest_in_range(p, metrics.glyph_start..metrics.glyph_end, &self.glyph_rects);
        }
    }

    fn key(&mut self, k: &Key, text: String) -> String {
        match k {
            Key::ArrowLeft => {
                self.back();
                text
            }
            Key::ArrowRight => {
                self.fwd(text.len());
                text
            }
            Key::ArrowUp => {
                self.up();
                text
            }
            Key::ArrowDown => {
                self.down();
                text
            }
            Key::Backspace => {
                if self.cursor > 0 {
                    let mut t = text;
                    t.remove(self.cursor - 1);
                    self.back();
                    t
                } else {
                    text
                }
            }
            Key::Character(c) => {
                let mut t = text;
                t.insert_str(self.cursor, &format!("{}", c));
                self.cursor += 1;
                t
            }
            Key::Space => {
                let mut t = text;
                t.insert(self.cursor, ' ');
                self.cursor += 1;
                t
            }
            Key::Home => {
                self.cursor = 0;
                text
            }
            Key::End => {
                self.cursor = text.len();
                text
            }
            _ => text,
        }
    }
}

impl TextEditorState {
    fn new() -> Self {
        Self {
            cursor: 0,
            glyph_rects: vec![],
            lines: vec![],
        }
    }
}

/// A multi-line text editor.
///
/// This shows how a complex View with internal
/// state can be created from more atomic Views.
pub fn text_editor(text: impl Binding<String>) -> impl View {
    focus(move |has_focus| {
        state(TextEditorState::new, move |state, _| {
            canvas(move |cx, rect, vger| {
                vger.translate([0.0, rect.height()]);
                let font_size = 18;
                let break_width = Some(rect.width());

                vger.text(text.get(cx), font_size, TEXT_COLOR, break_width);

                if has_focus {
                    let rects = vger.glyph_positions(text.get(cx), font_size, break_width);
                    let lines = vger.line_metrics(text.get(cx), font_size, break_width);
                    let glyph_rect_paint = vger.color_paint(vger::Color::MAGENTA);

                    cx[state].glyph_rects = rects;
                    cx[state].lines = lines;

                    let p = cx[state].cursor_pos();
                    vger.fill_rect(LocalRect::new(p, [2.0, 20.0].into()), 0.0, glyph_rect_paint);
                }
            })
            .key(move |cx, k| {
                if has_focus {
                    let t = text.with(cx, |t| t.clone());
                    let new_t = cx[state].key(&k, t);
                    text.with_mut(cx, |t| *t = new_t);
                }
            })
        })
    })
}
