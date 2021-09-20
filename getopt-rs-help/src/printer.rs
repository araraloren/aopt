use crate::style::Style;

use std::io::{Result, Write};

pub trait Printer<W: Write> {
    fn set_style(&mut self, style: Style);

    fn set_output_handle(&mut self, w: W);

    fn print_help(&mut self) -> Result<usize>;

    fn print_usage(&mut self) -> Result<usize>;

    fn print_header(&mut self) -> Result<usize>;

    fn print_footer(&mut self) -> Result<usize>;

    fn print_section_all(&mut self) -> Result<usize>;

    fn print_section(&mut self, section: &str) -> Result<usize>;

    fn print_cmd_usage(&mut self, cmd: &str) -> Result<usize>;

    fn print_cmd_header(&mut self, cmd: &str) -> Result<usize>;

    fn print_cmd_footer(&mut self, cmd: &str) -> Result<usize>;

    fn print_cmd_pos(&mut self, cmd: &str) -> Result<usize>;

    fn print_cmd_opt(&mut self, cmd: &str) -> Result<usize>;
}
