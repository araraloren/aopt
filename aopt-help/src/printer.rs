use crate::err::Result;
use crate::style::Style;

use std::io::Write;
use ustr::Ustr;

pub trait Printer<W: Write> {
    fn set_style(&mut self, style: Style);

    fn set_output_handle(&mut self, w: W);

    fn print_help(&mut self) -> Result<usize>;

    fn print_cmd_help(&mut self, cmd: Option<Ustr>) -> Result<usize>;

    fn print_section_all(&mut self) -> Result<usize>;

    fn print_section(&mut self, section: Ustr) -> Result<usize>;

    fn print_cmd_usage(&mut self, cmd: Option<Ustr>) -> Result<usize>;

    fn print_cmd_header(&mut self, cmd: Option<Ustr>) -> Result<usize>;

    fn print_cmd_footer(&mut self, cmd: Option<Ustr>) -> Result<usize>;

    fn print_cmd_pos(&mut self, cmd: Option<Ustr>) -> Result<usize>;

    fn print_cmd_opt(&mut self, cmd: Option<Ustr>) -> Result<usize>;
}
