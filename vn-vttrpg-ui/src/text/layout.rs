use crate::TextMetrics;
use vn_vttrpg_window::Glyph;

pub struct LaidOutLine {
    pub glyphs: Vec<Glyph>,
    pub width: f32,
    pub height: f32,
    pub char_start: usize,
    pub char_end: usize,
}

pub struct TextLayout {
    pub lines: Vec<LaidOutLine>,
    pub total_width: f32,
    pub total_height: f32,
}

impl TextLayout {
    pub fn layout(
        text: &str,
        font: &str,
        font_size: f32,
        max_width: f32,
        text_metrics: &dyn TextMetrics,
    ) -> Self {
        let line_height = text_metrics.line_height(font, font_size);
        let mut lines = Vec::new();
        let mut total_width: f32 = 0.0;
        let mut char_offset = 0;

        for (p_idx, paragraph) in text.split('\n').enumerate() {
            if p_idx > 0 {
                char_offset += 1; // for the newline character
            }

            if paragraph.is_empty() {
                lines.push(LaidOutLine {
                    glyphs: Vec::new(),
                    width: 0.0,
                    height: line_height,
                    char_start: char_offset,
                    char_end: char_offset,
                });
                continue;
            }

            let words: Vec<&str> = paragraph.split_inclusive(' ').collect();
            let mut current_line_glyphs = Vec::new();
            let mut current_line_width = 0.0;
            let mut line_char_start = char_offset;
            let mut current_char_offset = char_offset;

            for word in words {
                let word_glyphs = text_metrics.get_glyphs(word, font, font_size);
                let word_width: f32 = word_glyphs.iter().map(|g| g.advance).sum();
                let word_char_count = word.chars().count();

                if !current_line_glyphs.is_empty() && current_line_width + word_width > max_width {
                    // Start new line
                    lines.push(LaidOutLine {
                        glyphs: current_line_glyphs,
                        width: current_line_width,
                        height: line_height,
                        char_start: line_char_start,
                        char_end: current_char_offset,
                    });
                    total_width = total_width.max(current_line_width);
                    current_line_glyphs = word_glyphs;
                    current_line_width = word_width;
                    line_char_start = current_char_offset;
                } else {
                    current_line_glyphs.extend(word_glyphs);
                    current_line_width += word_width;
                }
                current_char_offset += word_char_count;
            }

            if !current_line_glyphs.is_empty() {
                lines.push(LaidOutLine {
                    glyphs: current_line_glyphs,
                    width: current_line_width,
                    height: line_height,
                    char_start: line_char_start,
                    char_end: current_char_offset,
                });
                total_width = total_width.max(current_line_width);
            }
            char_offset = current_char_offset;
        }

        let total_height = lines.len() as f32 * line_height;

        Self {
            lines,
            total_width,
            total_height,
        }
    }
}
