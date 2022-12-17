#[derive(Debug, Clone)]
pub enum Align {
    Left,
    Right,
}

impl Default for Align {
    fn default() -> Self {
        Self::Left
    }
}

#[derive(Debug, Clone)]
pub struct Style {
    pub align: Align,

    pub indent: usize,

    pub padding_char: char,

    pub wrap_width: usize,

    pub row_spacing: usize,

    pub line_spacing: usize,

    pub block_spacing: usize,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            align: Align::default(),
            indent: 2,
            padding_char: ' ',
            wrap_width: 0,
            row_spacing: 4,
            line_spacing: 0,
            block_spacing: 1,
        }
    }
}

impl Style {
    pub fn take(&mut self) -> Self {
        std::mem::take(self)
    }
}
