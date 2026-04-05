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
                let had_selection = self.selection_range().is_some()
                    && self.selection_range().unwrap().0 != self.selection_range().unwrap().1;
                let t = self.delete_selection(text);
                if !had_selection && self.cursor > 0 {
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
                let had_selection = self.selection_range().is_some()
                    && self.selection_range().unwrap().0 != self.selection_range().unwrap().1;
                let t = self.delete_selection(text);
                if !had_selection && self.cursor < t.len() {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_state(cursor: usize, text_len: usize) -> TextEditorState {
        // Create glyph rects for a single line of text, each char 10px wide, 20px tall
        let glyph_rects: Vec<LocalRect> = (0..text_len)
            .map(|i| LocalRect::new([i as f32 * 10.0, 0.0].into(), [10.0, 20.0].into()))
            .collect();
        let lines = vec![LineMetrics {
            glyph_start: 0,
            glyph_end: text_len,
            bounds: LocalRect::new(LocalPoint::zero(), [text_len as f32 * 10.0, 20.0].into()),
        }];
        TextEditorState {
            cursor,
            selection_start: None,
            glyph_rects,
            lines,
        }
    }

    fn make_multiline_state(cursor: usize, line_lengths: &[usize]) -> TextEditorState {
        let mut glyph_rects = vec![];
        let mut lines = vec![];
        let mut glyph_start = 0;
        for (line_idx, &len) in line_lengths.iter().enumerate() {
            let y = -(line_idx as f32) * 20.0;
            for i in 0..len {
                glyph_rects.push(LocalRect::new(
                    [i as f32 * 10.0, y].into(),
                    [10.0, 20.0].into(),
                ));
            }
            lines.push(LineMetrics {
                glyph_start,
                glyph_end: glyph_start + len,
                bounds: LocalRect::new([0.0, y].into(), [len as f32 * 10.0, 20.0].into()),
            });
            glyph_start += len;
        }
        TextEditorState {
            cursor,
            selection_start: None,
            glyph_rects,
            lines,
        }
    }

    #[test]
    fn test_cursor_pos_middle() {
        let s = make_state(2, 5);
        let p = s.cursor_pos();
        assert_eq!(p.x, 20.0);
        assert_eq!(p.y, 0.0);
    }

    #[test]
    fn test_cursor_pos_at_end() {
        let s = make_state(5, 5);
        let p = s.cursor_pos();
        // At end: last glyph origin.x + width
        assert_eq!(p.x, 50.0);
        assert_eq!(p.y, 0.0);
    }

    #[test]
    fn test_cursor_pos_empty() {
        let s = TextEditorState::new();
        let p = s.cursor_pos();
        assert_eq!(p.x, 0.0);
        assert_eq!(p.y, -20.0);
    }

    #[test]
    fn test_fwd_and_back() {
        let mut s = make_state(0, 5);
        s.fwd(5, false);
        assert_eq!(s.cursor, 1);
        s.fwd(5, false);
        assert_eq!(s.cursor, 2);
        s.back(false);
        assert_eq!(s.cursor, 1);
        s.back(false);
        assert_eq!(s.cursor, 0);
        // Can't go before 0
        s.back(false);
        assert_eq!(s.cursor, 0);
    }

    #[test]
    fn test_fwd_clamps_at_end() {
        let mut s = make_state(5, 5);
        s.fwd(5, false);
        assert_eq!(s.cursor, 5);
    }

    #[test]
    fn test_selection_with_shift() {
        let mut s = make_state(2, 5);
        // Move right with shift to start selection
        s.fwd(5, true);
        assert_eq!(s.cursor, 3);
        assert_eq!(s.selection_start, Some(2));
        assert_eq!(s.selection_range(), Some((2, 3)));

        // Extend selection
        s.fwd(5, true);
        assert_eq!(s.cursor, 4);
        assert_eq!(s.selection_range(), Some((2, 4)));

        // Move without shift clears selection
        s.fwd(5, false);
        assert_eq!(s.cursor, 5);
        assert_eq!(s.selection_start, None);
    }

    #[test]
    fn test_selection_backwards() {
        let mut s = make_state(3, 5);
        s.back(true);
        assert_eq!(s.cursor, 2);
        assert_eq!(s.selection_start, Some(3));
        // selection_range normalizes: start <= end
        assert_eq!(s.selection_range(), Some((2, 3)));
    }

    #[test]
    fn test_delete_selection() {
        let mut s = make_state(1, 5);
        s.selection_start = Some(1);
        s.cursor = 3;
        let result = s.delete_selection("abcde".to_string());
        assert_eq!(result, "ade");
        assert_eq!(s.cursor, 1);
        assert_eq!(s.selection_start, None);
    }

    #[test]
    fn test_delete_empty_selection() {
        let mut s = make_state(2, 5);
        s.selection_start = Some(2);
        s.cursor = 2;
        let result = s.delete_selection("abcde".to_string());
        // start == end, so nothing deleted
        assert_eq!(result, "abcde");
    }

    #[test]
    fn test_key_character() {
        let mut s = make_state(2, 5);
        let result = s.key(&Key::Character('x'), "abcde".to_string(), false);
        assert_eq!(result, "abxcde");
        assert_eq!(s.cursor, 3);
    }

    #[test]
    fn test_key_space() {
        let mut s = make_state(0, 3);
        let result = s.key(&Key::Space, "abc".to_string(), false);
        assert_eq!(result, " abc");
        assert_eq!(s.cursor, 1);
    }

    #[test]
    fn test_key_backspace() {
        let mut s = make_state(3, 5);
        let result = s.key(&Key::Backspace, "abcde".to_string(), false);
        assert_eq!(result, "abde");
        assert_eq!(s.cursor, 2);
    }

    #[test]
    fn test_key_backspace_at_start() {
        let mut s = make_state(0, 3);
        let result = s.key(&Key::Backspace, "abc".to_string(), false);
        assert_eq!(result, "abc");
        assert_eq!(s.cursor, 0);
    }

    #[test]
    fn test_key_backspace_with_selection() {
        let mut s = make_state(0, 5);
        s.selection_start = Some(1);
        s.cursor = 4;
        let result = s.key(&Key::Backspace, "abcde".to_string(), false);
        assert_eq!(result, "ae");
        assert_eq!(s.cursor, 1);
    }

    #[test]
    fn test_key_delete() {
        let mut s = make_state(2, 5);
        let result = s.key(&Key::Delete, "abcde".to_string(), false);
        assert_eq!(result, "abde");
        assert_eq!(s.cursor, 2);
    }

    #[test]
    fn test_key_delete_at_end() {
        let mut s = make_state(3, 3);
        let result = s.key(&Key::Delete, "abc".to_string(), false);
        assert_eq!(result, "abc");
        assert_eq!(s.cursor, 3);
    }

    #[test]
    fn test_key_home_end() {
        let mut s = make_state(3, 5);
        let result = s.key(&Key::Home, "abcde".to_string(), false);
        assert_eq!(result, "abcde");
        assert_eq!(s.cursor, 0);

        s.key(&Key::End, result, false);
        assert_eq!(s.cursor, 5);
    }

    #[test]
    fn test_key_home_with_shift() {
        let mut s = make_state(3, 5);
        s.key(&Key::Home, "abcde".to_string(), true);
        assert_eq!(s.cursor, 0);
        assert_eq!(s.selection_start, Some(3));
        assert_eq!(s.selection_range(), Some((0, 3)));
    }

    #[test]
    fn test_key_character_replaces_selection() {
        let mut s = make_state(0, 5);
        s.selection_start = Some(1);
        s.cursor = 3;
        let result = s.key(&Key::Character('x'), "abcde".to_string(), false);
        assert_eq!(result, "axde");
        assert_eq!(s.cursor, 2);
    }

    #[test]
    fn test_arrow_keys() {
        let mut s = make_state(2, 5);
        let text = "abcde".to_string();

        let result = s.key(&Key::ArrowLeft, text, false);
        assert_eq!(s.cursor, 1);

        let result = s.key(&Key::ArrowRight, result, false);
        assert_eq!(s.cursor, 2);
        assert_eq!(result, "abcde");
    }

    #[test]
    fn test_up_down_multiline() {
        // Two lines: "abc" (3 chars) and "defgh" (5 chars)
        let mut s = make_multiline_state(1, &[3, 5]);
        // cursor at index 1 in first line, move down
        s.down(false);
        // Should move to closest glyph in second line
        assert!(s.cursor >= 3 && s.cursor < 8);

        // Move back up
        s.up(false);
        assert!(s.cursor < 3);
    }

    #[test]
    fn test_up_at_first_line_stays() {
        let mut s = make_multiline_state(1, &[3, 5]);
        s.up(false);
        // Already on first line, cursor shouldn't change
        assert_eq!(s.cursor, 1);
    }

    #[test]
    fn test_down_at_last_line_stays() {
        let mut s = make_multiline_state(4, &[3, 5]);
        s.down(false);
        // Already on last line, cursor shouldn't change
        assert_eq!(s.cursor, 4);
    }

    #[test]
    fn test_find_line() {
        let s = make_multiline_state(0, &[3, 5, 2]);
        // cursor 0 is on line 0
        assert_eq!(s.find_line(), 0);

        let s2 = make_multiline_state(4, &[3, 5, 2]);
        // cursor 4 is on line 1 (glyphs 3..8)
        assert_eq!(s2.find_line(), 1);

        let s3 = make_multiline_state(9, &[3, 5, 2]);
        // cursor 9 is on line 2 (glyphs 8..10)
        assert_eq!(s3.find_line(), 2);
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
