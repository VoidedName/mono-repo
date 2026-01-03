use crate::TextMetrics;
use vn_window::Glyph;

#[derive(Clone)]
pub enum LineTermination {
    Newline,
    WordWrap,
}

#[derive(Clone)]
pub struct LaidOutLine {
    /// Glyphs that make up this line. The last glyph is the one that ends the line.
    /// unless [Self::terminator] is [LineTermination:Newline], in which case the newline
    /// character is not contained here as there is no rendered glyph for it.
    pub glyphs: Vec<Glyph>,
    pub width: f32,
    pub height: f32,
    /// First character index of this line
    pub char_start: usize,
    /// Last character index of this line (exclusive). If [Self::terminator] is [LineTermination::Newline],
    /// this is the index of the newline character (not contained in glyphs).
    pub char_end: usize,
    pub terminator: LineTermination,
}

#[derive(Clone)]
pub struct TextLayout {
    pub lines: Vec<LaidOutLine>,
    pub total_width: f32,
    pub total_height: f32,
}

// Remark (optimization): We could precompute and cache locations of glyphs to allow for bin
// searching them / or use a different spacial index with hitboxes to find the target glyph /
// the glyph above with a direct buffer location to hitbox lookup.
// this only matters for very long text. But since we recompute the layout on every change
// it isn't really worth it, unless we also address that.

impl TextLayout {
    pub fn layout(
        text: &str,
        font: &str,
        font_size: f32,
        max_width: Option<f32>,
        text_metrics: &dyn TextMetrics,
    ) -> Self {
        let mut lines = vec![];

        let line_height = text_metrics.line_height(font, font_size);
        let glyphs = text_metrics.get_glyphs(text, font, font_size);

        assert_eq!(
            glyphs.len(),
            text.chars().count(),
            "Glyph count mismatch for text: {:?}",
            text
        );

        let mut last_space: Option<usize> = None;
        let mut current_line_glyphs = LaidOutLine {
            glyphs: vec![],
            width: 0.0,
            height: line_height,
            char_start: 0,
            char_end: 0,
            terminator: LineTermination::WordWrap,
        };
        let mut current_word = Vec::new();
        let mut current_word_width = 0.0;

        for (idx, (glyph, c)) in glyphs.into_iter().zip(text.chars()).enumerate() {
            match c {
                '\n' => {
                    current_line_glyphs.char_end = idx + 1;
                    current_line_glyphs.width += current_word_width;
                    current_line_glyphs.terminator = LineTermination::Newline;
                    current_line_glyphs.glyphs.append(&mut current_word);

                    lines.push(current_line_glyphs);

                    current_line_glyphs = LaidOutLine {
                        glyphs: vec![],
                        width: 0.0,
                        height: line_height,
                        char_start: idx + 1,
                        char_end: 0,
                        terminator: LineTermination::WordWrap,
                    };

                    current_word_width = 0.0;
                    last_space = None;
                }
                _ => {
                    if let Some(max_width) = max_width
                        && max_width
                            < current_line_glyphs.width + current_word_width + glyph.advance
                    {
                        // space to next line
                        if c == ' ' {
                            last_space = Some(idx);
                            current_word_width = 0.0;

                            current_line_glyphs.glyphs.append(&mut current_word);
                            current_line_glyphs.width += current_word_width;
                            current_line_glyphs.char_end = idx;
                            current_line_glyphs.terminator = LineTermination::WordWrap;
                            lines.push(current_line_glyphs);

                            current_line_glyphs = LaidOutLine {
                                width: glyph.advance,
                                glyphs: vec![glyph],
                                height: line_height,
                                char_start: idx,
                                char_end: 0,
                                terminator: LineTermination::WordWrap,
                            };
                        }
                        // move entire word to next line (if it fits)
                        else if let Some(last_space) = last_space.take()
                            && current_word_width + glyph.advance <= max_width
                        {
                            current_word_width += glyph.advance;

                            current_word.push(glyph);

                            current_line_glyphs.terminator = LineTermination::WordWrap;
                            current_line_glyphs.char_end = last_space + 1;

                            lines.push(current_line_glyphs);

                            current_line_glyphs = LaidOutLine {
                                glyphs: vec![],
                                width: 0.0,
                                height: line_height,
                                char_start: last_space + 1,
                                char_end: 0,
                                terminator: LineTermination::WordWrap,
                            };
                        }
                        // break a word
                        else {
                            current_line_glyphs.terminator = LineTermination::WordWrap;
                            current_line_glyphs.char_end = idx;
                            current_line_glyphs.width += current_word_width;
                            current_word_width = glyph.advance;

                            current_line_glyphs.glyphs.append(&mut current_word);
                            lines.push(current_line_glyphs);

                            current_line_glyphs = LaidOutLine {
                                glyphs: vec![],
                                width: 0.0,
                                height: line_height,
                                char_start: idx,
                                char_end: 0,
                                terminator: LineTermination::WordWrap,
                            };
                            current_word.push(glyph);
                        }
                    } else {
                        // add a word to current line
                        if c == ' ' {
                            last_space = Some(idx);
                            current_line_glyphs.width += current_word_width + glyph.advance;
                            current_line_glyphs.char_end = idx + 1; // we don't really need to do this
                            current_line_glyphs.glyphs.append(&mut current_word);
                            current_line_glyphs.glyphs.push(glyph);

                            current_word_width = 0.0;
                        }
                        // extend current word
                        else {
                            current_word_width += glyph.advance;
                            current_word.push(glyph);
                        }
                    }
                }
            }
        }

        // last word is still missing from the line, but we know it fits, otherwise word wrap
        // would have happened before.
        if !current_word.is_empty() {
            current_line_glyphs.glyphs.append(&mut current_word);
            current_line_glyphs.width += current_word_width;
        }

        // last line is still missing from the layout
        current_line_glyphs.char_end = text.chars().count() + 1;
        current_line_glyphs.terminator = LineTermination::WordWrap;
        lines.push(current_line_glyphs);

        let total_width = lines
            .iter()
            .map(|l| l.width)
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0);
        let total_height = lines.iter().map(|l| l.height).sum();

        Self {
            lines,
            total_width,
            total_height,
        }
    }

    pub fn hit_test(&self, x: f32, y: f32) -> Option<usize> {
        if self.lines.is_empty() {
            return None;
        }

        // find candidate line click is inside of
        let mut line = None;
        {
            let mut total_y = 0.0;
            for candidate_line in self.lines.iter() {
                if y >= total_y && y < total_y + candidate_line.height {
                    line = Some(candidate_line);
                    break;
                }
                total_y += candidate_line.height;
            }
        }

        let line = match line {
            Some(line) => line,
            None => return None,
        };

        // find the glyph that the click is closer to
        let mut centered_glyphs = vec![0.0; line.glyphs.len()];
        {
            let mut glyph_x_offset = 0.0;
            for (i, glyph) in line.glyphs.iter().enumerate() {
                centered_glyphs[i] = glyph_x_offset + glyph.advance / 2.0;
                glyph_x_offset += glyph.advance;
            }
        }

        let hit = centered_glyphs.binary_search_by(|g_x| g_x.partial_cmp(&x).unwrap());

        match hit {
            Ok(idx) | Err(idx) => Some(line.char_start + idx),
        }
    }

    pub fn get_caret_pos(&self, char_index: usize) -> (f32, f32) {
        let mut current_y = 0.0;

        // Prefer the line where char_index is at the start (for wrapped lines)
        for line in &self.lines {
            if char_index == line.char_start {
                return (0.0, current_y);
            }
            current_y += line.height;
        }

        current_y = 0.0;
        let mut last_line = None;
        for line in &self.lines {
            if char_index >= line.char_start && char_index < line.char_end {
                let offset = char_index - line.char_start;
                let x = line.glyphs.iter().take(offset).map(|g| g.advance).sum();
                return (x, current_y);
            }
            last_line = Some((line, current_y));
            current_y += line.height;
        }

        if let Some((line, y)) = last_line {
            (line.width, y)
        } else {
            (0.0, 0.0)
        }
    }

    pub fn get_caret_x(&self, char_index: usize) -> f32 {
        self.get_caret_pos(char_index).0
    }

    pub fn get_vertical_move(&self, current_pos: usize, delta: i32, intended_x: f32) -> usize {
        if self.lines.is_empty() {
            return 0;
        }

        let mut current_line_idx = None;
        for (i, line) in self.lines.iter().enumerate() {
            if current_pos >= line.char_start && current_pos < line.char_end {
                current_line_idx = Some(i);
                break;
            }
        }

        let current_line_idx = current_line_idx.unwrap_or(self.lines.len() - 1);
        let target_line_idx =
            (current_line_idx as i32 + delta).clamp(0, self.lines.len() as i32 - 1) as usize;

        if target_line_idx == current_line_idx {
            return current_pos;
        }

        let target_line = &self.lines[target_line_idx];
        let mut glyph_x = 0.0;

        for (i, glyph) in target_line.glyphs.iter().enumerate() {
            let next_glyph_x = glyph_x + glyph.advance;

            if intended_x >= glyph_x && intended_x < next_glyph_x {
                return if intended_x - glyph_x < next_glyph_x - intended_x {
                    target_line.char_start + i
                } else {
                    target_line.char_start + i + 1
                };
            }
            glyph_x = next_glyph_x;
        }

        // x out of glyph bounds
        if intended_x < 0.0 {
            target_line.char_start
        } else {
            // we always want to be at the end of the text, which is before the newline / wrap
            target_line.char_end - 1
        }
    }
}
