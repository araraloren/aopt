
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
}

impl Default for Style {
    fn default() -> Self {
        Self {
            align: Alignment::default(),
            indent: 0,
            padding_char: ' ',
            wrap_width: 0,
        }
    }
}

impl Style {
    pub fn take(&mut self) -> Self {
        let old_v = std::mem::take(self);
        old_v
    }
}