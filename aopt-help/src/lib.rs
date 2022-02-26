//! Generate help message for aopt.
//!
//! # Example
//!
//! See the example code of [`snowball`](https://github.com/araraloren/aopt/blob/main/snowball-follow/src/main.rs#L384).
//!
//! ```ignore
//! fn simple_help_generate(set: &dyn Set) -> AppHelp<Stdout, DefaultFormat> {
//!     let mut help = AppHelp::default();
//!
//!     help.set_name("snowball".into());
//!
//!     let global = help.store.get_global_mut();
//!
//!     for opt in set.opt_iter() {
//!         if opt.match_style(aopt::opt::Style::Pos) {
//!             global.add_pos(PosStore::new(
//!                 opt.get_name(),
//!                 opt.get_hint(),
//!                 opt.get_help(),
//!                 opt.get_index().unwrap().to_string().into(),
//!                 opt.get_optional(),
//!             ));
//!         } else if !opt.match_style(aopt::opt::Style::Main) {
//!             global.add_opt(OptStore::new(
//!                 opt.get_name(),
//!                 opt.get_hint(),
//!                 opt.get_help(),
//!                 opt.get_type_name(),
//!                 opt.get_optional(),
//!             ));
//!         }
//!     }
//!
//!     global.set_header(gstr("Get the follow people number in https://xueqiu.com/"));
//!     global.set_footer(gstr(&format!("Create by araraloren {}", env!("CARGO_PKG_VERSION"))));
//!
//!     help
//! }
//! ```
//!
//!
mod err;
mod format;
mod printer;
mod store;
mod style;
mod wrapper;

use std::io::{Stdout, Write};
use ustr::Ustr;

pub use crate::err::Error;
pub use crate::err::Result;
pub use crate::format::Format;
pub use crate::printer::Printer;
pub use crate::store::CmdMut;
pub use crate::store::CmdOptMut;
pub use crate::store::CmdPosMut;
pub use crate::store::CmdStore;
pub use crate::store::OptStore;
pub use crate::store::PosStore;
pub use crate::store::SecStore;
pub use crate::store::SectionMut;
pub use crate::store::Store;
pub use crate::style::Alignment;
pub use crate::style::Style;
pub use crate::wrapper::Wrapped;
pub use crate::wrapper::Wrapper;

pub mod prelude {
    pub use crate::err::Error;
    pub use crate::err::Result;
    pub use crate::format::Format;
    pub use crate::printer::Printer;
    pub use crate::store::CmdMut;
    pub use crate::store::CmdOptMut;
    pub use crate::store::CmdPosMut;
    pub use crate::store::CmdStore;
    pub use crate::store::OptStore;
    pub use crate::store::PosStore;
    pub use crate::store::SecStore;
    pub use crate::store::SectionMut;
    pub use crate::store::Store;
    pub use crate::style::Alignment;
    pub use crate::style::Style;
    pub use crate::wrapper::Wrapped;
    pub use crate::wrapper::Wrapper;
    pub use crate::AppHelp;
    pub use crate::DefaultFormat;
    pub use std::io::Stdout;
    pub use std::io::Write;
    pub use ustr::Ustr;
}

#[derive(Debug)]
pub struct AppHelp<W: Write, F: Format> {
    name: Ustr,

    pub store: Store,

    style: Style,

    writer: W,

    format: F,
}

impl<W: Write, F: Format> AppHelp<W, F> {
    pub fn new(name: Ustr, style: Style, writer: W) -> Self {
        Self {
            name,
            store: Store::default(),
            style: style.clone(),
            writer,
            format: F::from(style),
        }
    }

    pub fn get_name(&self) -> Ustr {
        self.name
    }

    pub fn set_name(&mut self, name: Ustr) {
        self.name = name;
    }

    pub fn get_style(&self) -> &Style {
        &self.style
    }

    pub fn set_style(&mut self, style: Style) {
        self.style = style.clone();
        self.format.set_style(style);
    }

    pub fn set_writer(&mut self, writer: W) {
        self.writer = writer;
    }
}

impl<F: Format> Default for AppHelp<Stdout, F> {
    fn default() -> Self {
        Self {
            name: Ustr::default(),
            store: Store::default(),
            style: Style::default(),
            writer: std::io::stdout(),
            format: F::from(Style::default()),
        }
    }
}

impl<W: Write, F: Format> Printer<W> for AppHelp<W, F> {
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

    fn print_cmd_help(&mut self, cmd: Option<Ustr>) -> Result<usize> {
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
                if let Some(cmd_store) = self.store.get_cmd(*cmd_name) {
                    cmd_info.push(vec![
                        cmd_store.get_hint().as_ref(),
                        cmd_store.get_help().as_ref(),
                    ]);
                }
            }
            let mut buffer = String::new();

            buffer += self
                .format
                .format_sec_title(sec_store.get_help().as_ref())
                .as_ref();
            if !cmd_info.is_empty() {
                let mut wrapper = Wrapper::new(&cmd_info);

                wrapper.wrap();
                let wrapped = wrapper.get_output();

                for wrapped_line in wrapped {
                    let max_len = wrapped_line.iter().map(|v| v.len()).max().unwrap_or(1);

                    for i in 0..max_len {
                        buffer += self
                            .format
                            .format_sec_line(
                                &wrapped_line
                                    .iter()
                                    .map(|v| v.get_line(i))
                                    .collect::<Vec<String>>(),
                            )
                            .as_ref();
                        buffer += self.format.format_sec_new_line().as_ref();
                    }
                }
                buffer.truncate(buffer.len() - 1);
            }

            out += self.writer.write(buffer.as_bytes())?;
            out += self.writer.write(format!("\n").as_bytes())?;
        }
        Ok(out)
    }

    fn print_section(&mut self, section: Ustr) -> Result<usize> {
        let mut cmd_info = vec![];
        let sec_store = self
            .store
            .get_sec(section)
            .ok_or(Error::InvalidSecName(section.to_string()))?;

        for cmd_name in sec_store.cmd_iter() {
            if let Some(cmd_store) = self.store.get_cmd(*cmd_name) {
                cmd_info.push(vec![
                    cmd_store.get_hint().as_ref(),
                    cmd_store.get_help().as_ref(),
                ]);
            }
        }
        let mut buffer = String::new();

        buffer += self
            .format
            .format_sec_title(sec_store.get_help().as_ref())
            .as_ref();
        if !cmd_info.is_empty() {
            let mut wrapper = Wrapper::new(&cmd_info);

            wrapper.wrap();
            let wrapped = wrapper.get_output();

            for wrapped_line in wrapped {
                let max_len = wrapped_line.iter().map(|v| v.len()).max().unwrap_or(1);

                for i in 0..max_len {
                    buffer += self
                        .format
                        .format_sec_line(
                            &wrapped_line
                                .iter()
                                .map(|v| v.get_line(i))
                                .collect::<Vec<String>>(),
                        )
                        .as_ref();
                    buffer += self.format.format_sec_new_line().as_ref();
                }
            }
            buffer.truncate(buffer.len() - 1);
        }

        Ok(self.writer.write(buffer.as_bytes())?)
    }

    fn print_cmd_usage(&mut self, cmd: Option<Ustr>) -> Result<usize> {
        let mut buffer = String::new();
        let cmd_store = if let Some(cmd_name) = cmd {
            self.store
                .get_cmd(cmd_name)
                .ok_or(Error::InvalidCmdName(cmd_name.to_string()))?
        } else {
            self.store.get_global()
        };

        buffer += self
            .format
            .format_usage_name(self.get_name().as_ref())
            .as_ref();
        // for global cmd, print global option before <COMMAND>
        if cmd.is_none() {
            for opt_store in cmd_store.opt_iter() {
                buffer += self
                    .format
                    .format_usage_opt(opt_store.get_hint().as_ref(), opt_store.get_optional())
                    .as_ref();
            }
            if self.store.cmd_iter().len() > 0 {
                buffer += self.format.format_usage_cmd(None).as_ref();
            }
            buffer += self
                .format
                .format_usage_pos(
                    cmd_store.pos_len() + self.store.cmd_iter().fold(0, |acc, x| acc + x.pos_len()),
                )
                .as_ref();
        } else {
            // for any CMD, print option after it
            buffer += self
                .format
                .format_usage_cmd(Some(cmd_store.get_name().as_ref()))
                .as_ref();
            for opt_store in cmd_store.opt_iter() {
                buffer += self
                    .format
                    .format_usage_opt(opt_store.get_hint().as_ref(), opt_store.get_optional())
                    .as_ref();
            }
            buffer += self.format.format_usage_pos(cmd_store.pos_len()).as_ref();
        }
        buffer += "\n";
        Ok(self.writer.write(buffer.as_bytes())?)
    }

    fn print_cmd_header(&mut self, cmd: Option<Ustr>) -> Result<usize> {
        let cmd_store = if let Some(cmd_name) = cmd {
            self.store
                .get_cmd(cmd_name)
                .ok_or(Error::InvalidCmdName(cmd_name.to_string()))?
        } else {
            self.store.get_global()
        };
        let header = cmd_store.get_header();
        if header.is_empty() {
            Ok(0)
        } else {
            Ok(self
                .writer
                .write(self.format.format_header(header).as_bytes())?)
        }
    }

    fn print_cmd_footer(&mut self, cmd: Option<Ustr>) -> Result<usize> {
        let cmd_store = if let Some(cmd_name) = cmd {
            self.store
                .get_cmd(cmd_name)
                .ok_or(Error::InvalidCmdName(cmd_name.to_string()))?
        } else {
            self.store.get_global()
        };
        let footer = cmd_store.get_footer();
        if footer.is_empty() {
            Ok(0)
        } else {
            Ok(self
                .writer
                .write(self.format.format_footer(footer).as_bytes())?)
        }
    }

    fn print_cmd_pos(&mut self, cmd: Option<Ustr>) -> Result<usize> {
        let mut pos_info = vec![];
        let cmd_store = if let Some(cmd_name) = cmd {
            self.store
                .get_cmd(cmd_name)
                .ok_or(Error::InvalidCmdName(cmd_name.to_string()))?
        } else {
            self.store.get_global()
        };

        for pos_store in cmd_store.pos_iter() {
            pos_info.push(vec![
                pos_store.get_hint().as_ref(),
                pos_store.get_help().as_ref(),
            ]);
        }
        let mut buffer = String::new();

        if !pos_info.is_empty() {
            buffer += self.format.get_pos_title().as_ref();

            let mut wrapper = Wrapper::new(&pos_info);

            wrapper.wrap();
            let wrapped = wrapper.get_output();

            for wrapped_line in wrapped {
                let max_len = wrapped_line.iter().map(|v| v.len()).max().unwrap_or(1);

                for i in 0..max_len {
                    buffer += self
                        .format
                        .format_pos_line(
                            &wrapped_line
                                .iter()
                                .map(|v| v.get_line(i))
                                .collect::<Vec<String>>(),
                        )
                        .as_ref();
                    buffer += self.format.format_pos_new_line().as_ref();
                }
            }
        }

        Ok(self.writer.write(buffer.as_bytes())?)
    }

    fn print_cmd_opt(&mut self, cmd: Option<Ustr>) -> Result<usize> {
        let mut opt_info = vec![];
        let cmd_store = if let Some(cmd_name) = cmd {
            self.store
                .get_cmd(cmd_name)
                .ok_or(Error::InvalidCmdName(cmd_name.to_string()))?
        } else {
            self.store.get_global()
        };

        for opt_store in cmd_store.opt_iter() {
            opt_info.push(vec![
                opt_store.get_hint().as_ref(),
                opt_store.get_type_name().as_ref(),
                opt_store.get_help().as_ref(),
            ]);
        }
        let mut buffer = String::new();

        if !opt_info.is_empty() {
            buffer += self.format.get_opt_title().as_ref();

            let mut wrapper = Wrapper::new(&opt_info);

            wrapper.wrap();
            let wrapped = wrapper.get_output();

            for wrapped_line in wrapped {
                let max_len = wrapped_line.iter().map(|v| v.len()).max().unwrap_or(1);

                for i in 0..max_len {
                    buffer += self
                        .format
                        .format_opt_line(
                            &wrapped_line
                                .iter()
                                .map(|v| v.get_line(i))
                                .collect::<Vec<String>>(),
                        )
                        .as_ref();
                    buffer += self.format.format_opt_new_line().as_ref();
                }
            }
        }

        Ok(self.writer.write(buffer.as_bytes())?)
    }
}

#[derive(Debug, Clone, Default)]
pub struct DefaultFormat(Style);

impl Format for DefaultFormat {
    fn current_style(&self) -> &Style {
        &self.0
    }

    fn set_style(&mut self, style: Style) {
        self.0 = style;
    }
}

impl From<Style> for DefaultFormat {
    fn from(s: Style) -> Self {
        Self(s)
    }
}
