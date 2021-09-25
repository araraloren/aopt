pub mod printer;
pub mod store;
pub mod style;
pub mod wrapper;

use std::io::Result;
use std::io::{Stdout, Write};

use printer::Printer;
use store::Store;
use style::Style;

#[derive(Debug)]
pub struct AppHelp<W: Write> {
    name: String,

    pub store: Store,

    style: Style,

    writer: W,
}

impl<W: Write> AppHelp<W> {
    pub fn new(name: String, style: Style, writer: W) -> Self {
        Self {
            name,
            store: Store::default(),
            style,
            writer,
        }
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn get_style(&self) -> &Style {
        &self.style
    }

    pub fn set_style(&mut self, style: Style) {
        self.style = style;
    }

    pub fn set_writer(&mut self, writer: W) {
        self.writer = writer;
    }
}

impl Default for AppHelp<Stdout> {
    fn default() -> Self {
        Self {
            name: String::default(),
            store: Store::default(),
            style: Style::default(),
            writer: std::io::stdout(),
        }
    }
}

impl<W: Write> Printer<W> for AppHelp<W> {
    fn set_style(&mut self, style: Style) {
        self.style = style;
    }

    fn set_output_handle(&mut self, w: W) {
        self.writer = w;
    }

    fn print_help(&mut self) -> Result<usize> {
        todo!()
    }

    fn print_usage(&mut self) -> Result<usize> {
        let mut buffer = String::new();

        buffer += &format!("usage: {} ", self.get_name());
        for opt_store in self.store.get_global().opt_iter() {
            buffer += &format!("{} ", opt_store.get_hint());
        }
        if self.store.cmd_len() > 0 {
            buffer += &format!("<COMMAND> [args]\n");
        }
        self.writer.write(buffer.as_bytes())
    }

    fn print_header(&mut self) -> Result<usize> {
        self.writer
            .write(format!("{}\n", self.store.get_global().get_header()).as_bytes())
    }

    fn print_footer(&mut self) -> Result<usize> {
        self.writer
            .write(format!("{}\n", self.store.get_global().get_footer()).as_bytes())
    }

    fn print_pos(&mut self) -> Result<usize> {
        todo!()
    }

    fn print_opt(&mut self) -> Result<usize> {
        todo!()
    }

    fn print_section_all(&mut self) -> Result<usize> {
        todo!()
    }

    fn print_section(&mut self, section: &str) -> Result<usize> {
        todo!()
    }

    fn print_cmd_usage(&mut self, cmd: &str) -> Result<usize> {
        todo!()
    }

    fn print_cmd_header(&mut self, cmd: &str) -> Result<usize> {
        if let Some(cmd_store) = self.store.get_cmd(cmd) {
            self.writer
                .write(format!("{}\n", cmd_store.get_header()).as_bytes())
        } else {
            Ok(0)
        }
    }

    fn print_cmd_footer(&mut self, cmd: &str) -> Result<usize> {
        if let Some(cmd_store) = self.store.get_cmd(cmd) {
            self.writer
                .write(format!("{}\n", cmd_store.get_footer()).as_bytes())
        } else {
            Ok(0)
        }
    }

    fn print_cmd_pos(&mut self, cmd: &str) -> Result<usize> {
        todo!()
    }

    fn print_cmd_opt(&mut self, cmd: &str) -> Result<usize> {
        todo!()
    }
}
