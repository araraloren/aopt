use crate::style::Style;

pub trait Format: From<Style> {
    fn set_style(&mut self, style: Style);

    fn current_style(&self) -> &Style;

    fn format_usage_name(&self, name: &str) -> String {
        format!("usage: {}", name)
    }

    fn format_usage_opt(&self, hint: &str, optional: bool) -> String {
        if optional {
            format!("[{}]", hint)
        }
        else {
            format!("<{}>", hint)
        }
    }

    fn format_usage_cmd(&self, cmd_count: usize, pos_count: usize) -> String {
        let mut ret = String::default();

        if cmd_count > 0 {
            ret += "<COMMAND> ";
        }
        if pos_count > 0 {
            ret += "**ARGS**";
        }
        ret += "\n";
        ret
    }

    fn get_opt_title(&self) -> String {
        String::from("\nOPT:\n")
    }

    fn format_opt_line(&self, rows: &Vec<String>) -> String {
        rows.join(&" ".repeat(self.current_style().row_spacing))
    }

    fn format_opt_new_line(&self) -> String {
        format!("\n{}", "\n".repeat(self.current_style().opt_line_spacing))
    }

    fn get_pos_title(&self) -> String {
        String::from("\nPOS:\n")
    }

    fn format_pos_line(&self, rows: &Vec<String>) -> String {
        rows.join(&" ".repeat(self.current_style().row_spacing))
    }

    fn format_pos_new_line(&self) -> String {
        format!("\n{}", "\n".repeat(self.current_style().pos_line_spacing))
    }

    fn format_sec_title(&self, help: &str) -> String {
        format!("\n{}\n", help)
    }

    fn format_sec_line(&self, rows: &Vec<String>) -> String {
        rows.join(&" ".repeat(self.current_style().row_spacing))
    }

    fn format_sec_new_line(&self) -> String {
        format!("\n{}", "\n".repeat(self.current_style().cmd_line_spacing))
    }

    fn format_footer(&self, footer: &str) -> String {
        format!("\n{}\n", footer)
    }

    fn format_header(&self, header: &str) -> String {
        format!("\n{}\n", header)
    }
}