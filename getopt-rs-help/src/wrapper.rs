use crate::style::{Alignment, Style};
use std::borrow::Cow;
use textwrap::{core::display_width, wrap};

#[derive(Debug, Default)]
pub struct Wrapped<'a> {
    cows: Vec<Cow<'a, str>>,

    style: Style,
}

impl<'a> Wrapped<'a> {
    pub fn new(cows: Vec<Cow<'a, str>>, style: Style) -> Self {
        Self { cows, style }
    }

    pub fn get_style(&self) -> &Style {
        &self.style
    }

    pub fn set_wrap_width(&mut self, width: usize) {
        self.style.wrap_width = width;
    }

    pub fn get_wrap_width(&self) -> usize {
        self.style.wrap_width
    }

    pub fn get_line(&self, line: usize) -> String {
        let padding_str = String::from(self.style.padding_char);

        if line < self.cows.len() {
            let mut ret = " ".repeat(self.style.indent);
            let real_width = display_width(self.cows[line].as_ref());
            let padding_width = self.get_wrap_width() - real_width;

            ret += self.cows[line].as_ref();
            match self.style.align {
                Alignment::Left => {
                    ret += &padding_str.repeat(padding_width);
                }
                Alignment::Right => {
                    ret = padding_str.repeat(padding_width) + &ret;
                }
            }
            ret
        } else {
            format!(
                "{}{}",
                " ".repeat(self.style.indent),
                padding_str.repeat(self.get_wrap_width())
            )
        }
    }

    pub fn len(&self) -> usize {
        self.cows.len()
    }
}

#[derive(Debug)]
pub struct Wrapper<'a, 'b> {
    data: &'a Vec<Vec<&'b str>>,

    output: Vec<Vec<Wrapped<'b>>>,
}

impl<'a, 'b> Wrapper<'a, 'b> {
    pub fn new(data: &'a Vec<Vec<&'b str>>) -> Self {
        Self {
            data,
            output: vec![],
        }
    }

    pub fn wrap(&mut self) {
        let data_len = self.data.iter().map(|v| v.len()).max().unwrap_or(0);
        let mut default_style = vec![Style::default(); data_len];

        for line in self.data.iter() {
            for (style_mut, col) in default_style.iter_mut().zip(line.iter()) {
                let width = display_width(col);
                if style_mut.wrap_width < width {
                    style_mut.wrap_width = width;
                }
            }
        }

        for line in self.data.iter() {
            let mut wrapped = vec![];

            for (col, style) in line.iter().zip(default_style.iter()) {
                wrapped.push(Wrapped::new(wrap(col, style.wrap_width), style.clone()));
            }

            self.output.push(wrapped);
        }
    }

    pub fn wrap_with(&mut self, styles: Vec<Style>) {
        for line in self.data.iter() {
            let mut wrapped = vec![];

            for (col, style) in line.iter().zip(styles.iter()) {
                wrapped.push(Wrapped::new(wrap(col, style.wrap_width), style.clone()));
            }
            self.output.push(wrapped);
        }
    }

    pub fn get_output(&self) -> &Vec<Vec<Wrapped<'b>>> {
        &self.output
    }

    pub fn len(&self) -> usize {
        self.output.len()
    }
}
