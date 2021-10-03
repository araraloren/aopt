pub mod printer;
pub mod store;
pub mod style;
pub mod wrapper;

use std::borrow::Cow;
use std::io::Result;
use std::io::{Stdout, Write};

use printer::Printer;
use store::Store;
use style::Style;

use crate::wrapper::Wrapper;

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
            if opt_store.get_optional() {
                buffer += &format!("[{}] ", opt_store.get_hint());
            }
            else {
                buffer += &format!("<{}> ", opt_store.get_hint());
            }
        }
        if self.store.cmd_len() > 0 {
            buffer += &format!("<COMMAND> ");
        }
        if self.store.get_global().pos_len() > 0 {
            buffer += &format!("**ARGS**");
        } else {
            for cmd_store in self.store.cmd_iter() {
                if cmd_store.pos_len() > 0 {
                    buffer += &format!("**ARGS**");
                    break;
                }
            }
        }
        buffer += "\n";
        self.writer.write(buffer.as_bytes())
    }

    fn print_header(&mut self) -> Result<usize> {
        let header = self.store.get_global().get_header();
        if header.is_empty() {
            Ok(0)
        } else {
            self.writer.write(format!("\n{}\n", header).as_bytes())
        }
    }

    fn print_footer(&mut self) -> Result<usize> {
        let footer = self.store.get_global().get_footer();
        if footer.is_empty() {
            Ok(0)
        } else {
            self.writer.write(format!("\n{}\n", footer).as_bytes())
        }
    }

    fn print_pos(&mut self) -> Result<usize> {
        let mut pos_info = vec![];

        for pos_store in self.store.get_global().pos_iter() {
            pos_info.push(vec![pos_store.get_hint(), pos_store.get_help()]);
        }
        let mut buffer = String::new();

        buffer += "\nPOS:\n";
        if !pos_info.is_empty() {
            let mut wrapper = Wrapper::new(&pos_info);

            wrapper.wrap();
            let wrapped = wrapper.get_output();

            for wrapped_line in wrapped {
                let max_len = wrapped_line.iter().map(|v| v.len()).max().unwrap_or(1);

                for i in 0..max_len {
                    for line in wrapped_line {
                        buffer += &line.get_line(i);
                    }
                    buffer += "\n\n";
                }
            }
            buffer.truncate(buffer.len() - 1);
        }

        self.writer.write(buffer.as_bytes())
    }

    fn print_opt(&mut self) -> Result<usize> {
        let mut opt_info = vec![];

        for opt_store in self.store.get_global().opt_iter() {
            opt_info.push(vec![opt_store.get_hint(), opt_store.get_help()]);
        }
        let mut buffer = String::new();

        buffer += "\nOPT:\n";
        if !opt_info.is_empty() {
            let mut wrapper = Wrapper::new(&opt_info);

            wrapper.wrap();
            let wrapped = wrapper.get_output();

            for wrapped_line in wrapped {
                let max_len = wrapped_line.iter().map(|v| v.len()).max().unwrap_or(1);

                for i in 0..max_len {
                    for line in wrapped_line {
                        buffer += &line.get_line(i);
                    }
                    buffer += "\n\n";
                }
            }
            buffer.truncate(buffer.len() - 1);
        }

        self.writer.write(buffer.as_bytes())
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
