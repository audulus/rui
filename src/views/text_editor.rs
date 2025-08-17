use crate::*;

/// View-model for `text_editor`.
struct TextEditorState {
    cursor: usize,
    selection_start: Option<usize>,
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

    /// Returns the current selection range (start, end) with start <= end
    fn selection_range(&self) -> Option<(usize, usize)> {
        if let Some(start) = self.selection_start {
            let (start, end) = if start <= self.cursor {
                (start, self.cursor)
            } else {
                (self.cursor, start)
            };
            Some((start, end))
        } else {
            None
        }
    }

    /// Clears the current selection
    fn clear_selection(&mut self) {
        self.selection_start = None;
    }

    /// Starts or extends a selection
    fn start_selection(&mut self) {
        if self.selection_start.is_none() {
            self.selection_start = Some(self.cursor);
        }
    }

    /// Deletes the selected text and returns the new text
    fn delete_selection(&mut self, text: String) -> String {
        if let Some((start, end)) = self.selection_range() {
            if start != end {
                let mut t = text;
                t.drain(start..end);
                self.cursor = start;
                self.clear_selection();
                return t;
            }
        }
        text
    }

    fn fwd(&mut self, len: usize, extend_selection: bool) {
        if extend_selection {
            self.start_selection();
        } else {
            self.clear_selection();
        }
        self.cursor += 1;
        if self.cursor > len {
            self.cursor = len;
        }
    }
    
    fn back(&mut self, extend_selection: bool) {
        if extend_selection {
            self.start_selection();
        } else {
            self.clear_selection();
        }
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

    fn down(&mut self, extend_selection: bool) {
        if extend_selection {
            self.start_selection();
        } else {
            self.clear_selection();
        }
        
        let p = self.cursor_pos();

        let line = self.find_line() + 1;
        if line < self.lines.len() {
            let metrics = self.lines[line];
            self.cursor =
                self.closest_in_range(p, metrics.glyph_start..metrics.glyph_end, &self.glyph_rects);
        }
    }

    fn up(&mut self, extend_selection: bool) {
        if extend_selection {
            self.start_selection();
        } else {
            self.clear_selection();
        }
        
        let p = self.cursor_pos();

        let line = self.find_line();
        if line > 0 {
            let metrics = self.lines[line - 1];
            self.cursor =
                self.closest_in_range(p, metrics.glyph_start..metrics.glyph_end, &self.glyph_rects);
        }
    }

    fn key(&mut self, k: &Key, text: String, shift_pressed: bool) -> String {
        
        match k {
            Key::ArrowLeft => {
                self.back(shift_pressed);
                text
            }
            Key::ArrowRight => {
                self.fwd(text.len(), shift_pressed);
                text
            }
            Key::ArrowUp => {
                self.up(shift_pressed);
                text
            }
            Key::ArrowDown => {
                self.down(shift_pressed);
                text
            }
            Key::Backspace => {
                // First try to delete selection
                let t = self.delete_selection(text);
                if self.selection_range().is_none() && self.cursor > 0 {
                    // No selection was deleted, do normal backspace
                    let mut t = t;
                    t.remove(self.cursor - 1);
                    self.back(false);
                    t
                } else {
                    t
                }
            }
            Key::Delete => {
                // Delete selection or character at cursor
                let t = self.delete_selection(text);
                if self.selection_range().is_none() && self.cursor < t.len() {
                    // No selection was deleted, delete character at cursor
                    let mut t = t;
                    t.remove(self.cursor);
                    t
                } else {
                    t
                }
            }
            Key::Character(c) => {
                // Replace selection or insert character
                let mut t = self.delete_selection(text);
                t.insert_str(self.cursor, &c.to_string());
                self.cursor += 1;
                t
            }
            Key::Space => {
                // Replace selection or insert space
                let mut t = self.delete_selection(text);
                t.insert(self.cursor, ' ');
                self.cursor += 1;
                t
            }
            Key::Home => {
                if shift_pressed {
                    self.start_selection();
                } else {
                    self.clear_selection();
                }
                self.cursor = 0;
                text
            }
            Key::End => {
                if shift_pressed {
                    self.start_selection();
                } else {
                    self.clear_selection();
                }
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
            selection_start: None,
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

                    cx[state].glyph_rects = rects;
                    cx[state].lines = lines;

                    // Render selection background
                    if let Some((start, end)) = cx[state].selection_range() {
                        if start != end {
                            let selection_paint = vger.color_paint(vger::Color::new(0.3, 0.6, 1.0, 0.3));
                            
                            for i in start..end.min(cx[state].glyph_rects.len()) {
                                let rect = cx[state].glyph_rects[i];
                                vger.fill_rect(
                                    LocalRect::new(rect.origin, [rect.size.width.max(2.0), 20.0].into()),
                                    0.0,
                                    selection_paint
                                );
                            }
                            
                            // Handle selection at end of text
                            if end >= cx[state].glyph_rects.len() && !cx[state].glyph_rects.is_empty() {
                                if let Some(last_rect) = cx[state].glyph_rects.last() {
                                    let end_pos = [last_rect.origin.x + last_rect.size.width, last_rect.origin.y];
                                    vger.fill_rect(
                                        LocalRect::new(end_pos.into(), [2.0, 20.0].into()),
                                        0.0,
                                        selection_paint
                                    );
                                }
                            }
                        }
                    }

                    // Render cursor
                    let cursor_paint = vger.color_paint(vger::Color::MAGENTA);
                    let p = cx[state].cursor_pos();
                    vger.fill_rect(LocalRect::new(p, [2.0, 20.0].into()), 0.0, cursor_paint);
                }
            })
            .key(move |cx, k| {
                if has_focus {
                    let t = text.with(cx, |t| t.clone());
                    let shift_pressed = cx.key_mods.shift;
                    let new_t = cx[state].key(&k, t, shift_pressed);
                    text.with_mut(cx, |t| *t = new_t);
                }
            })
        })
    })
}
