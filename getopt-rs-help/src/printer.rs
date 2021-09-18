use crate::{style::Style};

use std::io::Write;

pub trait Printer<W: Write> {
    fn set_style(&mut self, style: Style);

    fn set_output_handle(&mut self, w: &mut W);

    fn print_help(&self);

    fn print_usage(&self);

    fn print_header(&self);

    fn print_footer(&self);

    fn print_section_all(&self);

    fn print_section(&self, section: &str);

    fn print_cmd_usage(&self, cmd: &str);

    fn print_cmd_header(&self, cmd: &str);

    fn print_cmd_footer(&self, cmd: &str);

    fn print_cmd_pos(&self, cmd: &str);

    fn print_cmd_opt(&self, cmd: &str);
}
