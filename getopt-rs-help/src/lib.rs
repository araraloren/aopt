
pub mod store;
pub mod style;
pub mod wrapper;
pub mod printer;

use std::io::Write;

use printer::Printer;
use store::Store;
use style::Style;

pub struct AppHelp<'a, W: Write> {
    name: String,

    store: Store,

    style: Style,

    writer: &'a mut W,
}

impl<'a, W: std::io::Write> Printer<W> for AppHelp<'a, W> {
    fn set_style(&mut self, style: Style) {
        todo!()
    }

    fn set_output_handle(&mut self, w: &mut W) {
        todo!()
    }

    fn print_help(&self) {
        todo!()
    }

    fn print_usage(&self) {
        todo!()
    }

    fn print_header(&self) {
        todo!()
    }

    fn print_footer(&self) {
        todo!()
    }

    fn print_section_all(&self) {
        todo!()
    }

    fn print_section(&self, section: &str) {
        todo!()
    }

    fn print_cmd_usage(&self, cmd: &str) {
        todo!()
    }

    fn print_cmd_header(&self, cmd: &str) {
        todo!()
    }

    fn print_cmd_footer(&self, cmd: &str) {
        todo!()
    }

    fn print_cmd_pos(&self, cmd: &str) {
        todo!()
    }

    fn print_cmd_opt(&self, cmd: &str) {
        todo!()
    }
}