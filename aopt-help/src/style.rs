#[derive(Debug, Clone)]
pub enum Alignment {
    Left,
    Right,
}

impl Default for Alignment {
    fn default() -> Self {
        Self::Left
    }
}

#[derive(Debug, Clone)]
pub struct Style {
    pub align: Alignment,

    pub indent: usize,

    pub padding_char: char,

    pub wrap_width: usize,

    pub row_spacing: usize,

    pub opt_line_spacing: usize,

    pub pos_line_spacing: usize,

    pub cmd_line_spacing: usize,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            align: Alignment::default(),
            indent: 2,
            padding_char: ' ',
            wrap_width: 0,
            row_spacing: 4,
            opt_line_spacing: 0,
            pos_line_spacing: 0,
            cmd_line_spacing: 0,
        }
    }
}

impl Style {
    pub fn take(&mut self) -> Self {
        let old_v = std::mem::take(self);
        old_v
    }
}
