pub mod err;
pub mod printer;
pub mod store;
pub mod style;
pub mod wrapper;

use crate::err::{Error, Result};
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
        self.print_cmd_usage(None)?;
        self.print_cmd_header(None)?;
        self.print_section_all()?;
        self.print_cmd_footer(None)
    }

    fn print_cmd_help(&mut self, cmd: Option<&str>) -> Result<usize> {
        self.print_cmd_usage(cmd)?;
        self.print_cmd_header(cmd)?;
        self.print_cmd_pos(cmd)?;
        self.print_cmd_opt(cmd)?;
        self.print_cmd_footer(cmd)
    }

    fn print_section_all(&mut self) -> Result<usize> {
        let mut out: usize = 0;
        for sec_store in self.store.sec_iter() {
            let mut cmd_info = vec![];

            for cmd_name in sec_store.cmd_iter() {
                if let Some(cmd_store) = self.store.get_cmd(cmd_name) {
                    cmd_info.push(vec![cmd_store.get_hint(), cmd_store.get_help()]);
                }
            }
            let mut buffer = String::new();

            buffer += &format!("\n{}\n", sec_store.get_help());
            if !cmd_info.is_empty() {
                let mut wrapper = Wrapper::new(&cmd_info);

                wrapper.wrap();
                let wrapped = wrapper.get_output();

                for wrapped_line in wrapped {
                    let max_len = wrapped_line.iter().map(|v| v.len()).max().unwrap_or(1);

                    for i in 0..max_len {
                        buffer += &wrapped_line
                            .iter()
                            .map(|v| v.get_line(i))
                            .collect::<Vec<String>>()
                            .join(&" ".repeat(self.style.row_spacing));
                        buffer += &format!("\n{}", "\n".repeat(self.style.cmd_line_spacing));
                    }
                }
                buffer.truncate(buffer.len() - 1);
            }

            out += self.writer.write(buffer.as_bytes())?;
            out += self.writer.write(format!("\n").as_bytes())?;
        }
        Ok(out)
    }

    fn print_section(&mut self, section: &str) -> Result<usize> {
        let mut cmd_info = vec![];
        let sec_store = self
            .store
            .get_sec(section)
            .ok_or(Error::InvalidSecName(String::from(section)))?;

        for cmd_name in sec_store.cmd_iter() {
            if let Some(cmd_store) = self.store.get_cmd(cmd_name) {
                cmd_info.push(vec![cmd_store.get_hint(), cmd_store.get_help()]);
            }
        }
        let mut buffer = String::new();

        buffer += &format!("\n{}\n", sec_store.get_help());
        if !cmd_info.is_empty() {
            let mut wrapper = Wrapper::new(&cmd_info);

            wrapper.wrap();
            let wrapped = wrapper.get_output();

            for wrapped_line in wrapped {
                let max_len = wrapped_line.iter().map(|v| v.len()).max().unwrap_or(1);

                for i in 0..max_len {
                    buffer += &wrapped_line
                        .iter()
                        .map(|v| v.get_line(i))
                        .collect::<Vec<String>>()
                        .join(&" ".repeat(self.style.row_spacing));
                    buffer += &format!("\n{}", "\n".repeat(self.style.cmd_line_spacing));
                }
            }
            buffer.truncate(buffer.len() - 1);
        }

        Ok(self.writer.write(buffer.as_bytes())?)
    }

    fn print_cmd_usage(&mut self, cmd: Option<&str>) -> Result<usize> {
        let mut buffer = String::new();
        let cmd_store = if let Some(cmd_name) = cmd {
            self.store
                .get_cmd(cmd_name)
                .ok_or(Error::InvalidCmdName(String::from(cmd_name)))?
        } else {
            self.store.get_global()
        };

        buffer += &format!("usage: {} ", self.get_name());
        for opt_store in cmd_store.opt_iter() {
            if opt_store.get_optional() {
                buffer += &format!("[{}] ", opt_store.get_hint());
            } else {
                buffer += &format!("<{}> ", opt_store.get_hint());
            }
        }
        if self.store.cmd_len() > 0 {
            buffer += &format!("<COMMAND> ");
        }
        if cmd_store.pos_len() > 0 {
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
        Ok(self.writer.write(buffer.as_bytes())?)
    }

    fn print_cmd_header(&mut self, cmd: Option<&str>) -> Result<usize> {
        let cmd_store = if let Some(cmd_name) = cmd {
            self.store
                .get_cmd(cmd_name)
                .ok_or(Error::InvalidCmdName(String::from(cmd_name)))?
        } else {
            self.store.get_global()
        };
        let header = cmd_store.get_header();
        if header.is_empty() {
            Ok(0)
        } else {
            Ok(self.writer.write(format!("\n{}\n", header).as_bytes())?)
        }
    }

    fn print_cmd_footer(&mut self, cmd: Option<&str>) -> Result<usize> {
        let cmd_store = if let Some(cmd_name) = cmd {
            self.store
                .get_cmd(cmd_name)
                .ok_or(Error::InvalidCmdName(String::from(cmd_name)))?
        } else {
            self.store.get_global()
        };
        let footer = cmd_store.get_footer();
        if footer.is_empty() {
            Ok(0)
        } else {
            Ok(self.writer.write(format!("\n{}\n", footer).as_bytes())?)
        }
    }

    fn print_cmd_pos(&mut self, cmd: Option<&str>) -> Result<usize> {
        let mut pos_info = vec![];
        let cmd_store = if let Some(cmd_name) = cmd {
            self.store
                .get_cmd(cmd_name)
                .ok_or(Error::InvalidCmdName(String::from(cmd_name)))?
        } else {
            self.store.get_global()
        };

        for pos_store in cmd_store.pos_iter() {
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
                    buffer += &wrapped_line
                        .iter()
                        .map(|v| v.get_line(i))
                        .collect::<Vec<String>>()
                        .join(&" ".repeat(self.style.row_spacing));
                    buffer += &format!("\n{}", "\n".repeat(self.style.pos_line_spacing));
                }
            }
            buffer.truncate(buffer.len() - 1);
        }

        Ok(self.writer.write(buffer.as_bytes())?)
    }

    fn print_cmd_opt(&mut self, cmd: Option<&str>) -> Result<usize> {
        let mut opt_info = vec![];
        let cmd_store = if let Some(cmd_name) = cmd {
            self.store
                .get_cmd(cmd_name)
                .ok_or(Error::InvalidCmdName(String::from(cmd_name)))?
        } else {
            self.store.get_global()
        };

        for opt_store in cmd_store.opt_iter() {
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
                    buffer += &wrapped_line
                        .iter()
                        .map(|v| v.get_line(i))
                        .collect::<Vec<String>>()
                        .join(&" ".repeat(self.style.row_spacing));
                    buffer += &format!("\n{}", "\n".repeat(self.style.opt_line_spacing));
                }
            }
            buffer.truncate(buffer.len() - 1);
        }

        Ok(self.writer.write(buffer.as_bytes())?)
    }
}
