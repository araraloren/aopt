use crate::style::Style;

pub trait Format: From<Style> {
    fn set_style(&mut self, style: Style);

    fn current_style(&self) -> &Style;

    fn format_usage_name(&self, name: &str) -> String {
        format!("usage: {} ", name)
    }

    fn format_usage_opt(&self, hint: &str, optional: bool) -> String {
        if optional {
            format!("[{}] ", hint)
        } else {
            format!("<{}> ", hint)
        }
    }

    fn format_usage_cmd(&self, cmd: Option<&str>) -> String {
        if let Some(cmd_name) = cmd {
            format!("{} ", cmd_name)
        } else {
            String::from("<COMMAND> ")
        }
    }

    fn format_usage_pos(&self, pos_count: usize) -> String {
        if pos_count > 0 {
            String::from("**ARGS**")
        } else {
            String::default()
        }
    }

    fn get_opt_title(&self) -> String {
        String::from("\nOPT:\n")
    }

    fn format_opt_line(&self, rows: &[String]) -> String {
        rows.join(&" ".repeat(self.current_style().row_spacing))
    }

    fn format_opt_new_line(&self) -> String {
        format!("\n{}", "\n".repeat(self.current_style().opt_line_spacing))
    }

    fn get_pos_title(&self) -> String {
        String::from("\nPOS:\n")
    }

    fn format_pos_line(&self, rows: &[String]) -> String {
        rows.join(&" ".repeat(self.current_style().row_spacing))
    }

    fn format_pos_new_line(&self) -> String {
        format!("\n{}", "\n".repeat(self.current_style().pos_line_spacing))
    }

    fn format_sec_title(&self, help: &str) -> String {
        format!("\n{}\n", help)
    }

    fn format_sec_line(&self, rows: &[String]) -> String {
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
