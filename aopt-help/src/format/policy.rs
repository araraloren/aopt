use std::borrow::Cow;
use std::io::Write;
use std::marker::PhantomData;

use crate::block::Block;
use crate::cmd::Command;
use crate::store::Store;
use crate::style::Style;
use crate::wrapper::Wrapper;
use crate::AppHelp;
use crate::HelpPolicy;

// struct UsageDetail<'a> {
//     store_usages: Vec<Cow<'a, str>>,

//     args: Vec<Cow<'a, str>>,
// }

pub struct DefaultPolicy<'a, I> {
    name: Cow<'a, str>,

    style: Style,

    styles: Vec<Style>,

    max_width: usize,

    hiding_pos: bool,

    usage_new_line: usize,

    marker: PhantomData<&'a I>,
}

impl<I> Default for DefaultPolicy<'_, I> {
    fn default() -> Self {
        Self {
            name: Default::default(),
            style: Default::default(),
            styles: Default::default(),
            max_width: 0,
            hiding_pos: true,
            usage_new_line: 0,
            marker: Default::default(),
        }
    }
}

impl<'a, I> DefaultPolicy<'a, I> {
    pub fn new<S: Into<Cow<'a, str>>>(
        name: S,
        style: Style,
        block: Vec<Style>,
        max_width: usize,
        hiding_pos: bool,
        usage_new_line: usize,
    ) -> Self {
        Self {
            name: name.into(),
            style,
            styles: block,
            max_width,
            hiding_pos,
            usage_new_line,
            marker: PhantomData,
        }
    }
}

impl<'a> DefaultPolicy<'a, Command<'a>> {
    pub fn get_block_usage(
        &self,
        item: &Block<'a, Cow<'a, str>>,
        stores: &[Store<'a>],
    ) -> (Vec<String>, Vec<String>) {
        let mut usages = vec![];
        let mut args = vec![];

        for store in item.iter() {
            if let Some(store) = stores.iter().find(|v| &v.name() == store) {
                let hint = store.hint();

                if !hint.is_empty() {
                    if store.position() {
                        if store.optional() {
                            args.push(format!("[{}]", hint));
                        } else {
                            args.push(format!("<{}>", hint));
                        }
                    } else if store.optional() {
                        usages.push(format!("[{}]", hint));
                    } else {
                        usages.push(format!("<{}>", hint));
                    }
                }
            }
        }
        (usages, args)
    }

    pub fn get_command_usage(&self, item: &Command<'a>) -> Cow<'a, str> {
        let mut usages = vec![];
        let mut args = vec![];
        let mut block_hint = vec![];

        for block in item.block() {
            let (block_usages, mut block_args) = self.get_block_usage(block, item);

            if !block_usages.is_empty() {
                for mut usage in block_usages {
                    if self.usage_new_line > 0 && (usages.len() + 1) % self.usage_new_line == 0 {
                        // add more space
                        // same length as `Usage: `
                        usage.push_str("\n      ");
                    }
                    usages.push(usage);
                }
            }
            // if not omit args, using the args, otherwise using hint of block
            if !block_args.is_empty() {
                args.append(&mut block_args);
            }
        }
        for block in item.block() {
            if !block.is_empty() {
                let arg = block.hint();

                if !arg.is_empty() {
                    block_hint.push(arg);
                }
            }
        }

        let mut ret = String::from("Usage: ");
        let usage = usages.join(" ");
        let block_hint = block_hint.join(" ");
        let args = args.join(" ");

        if !self.name.is_empty() {
            ret += &self.name;
            ret += " ";
        }
        if !item.name().is_empty() {
            ret += &item.name();
            ret += " ";
        }
        if !usage.is_empty() {
            ret += &usage;
            ret += " ";
        }
        if self.hiding_pos {
            if !block_hint.is_empty() {
                ret += &block_hint;
                ret += " ";
            }
        } else if !args.is_empty() {
            ret += &args;
            ret += " ";
        }
        ret.into()
    }

    pub fn get_block_help(
        &self,
        item: &Block<'a, Cow<'a, str>>,
        stores: &Vec<Store<'a>>,
    ) -> Cow<'a, str> {
        let style = &self.style;
        let count = item.len();
        let head = item.head();
        let foot = item.foot();
        let line_spacing = &"\n".repeat(1 + style.line_spacing);
        let mut output = if head.is_empty() { vec![] } else { vec![head] };
        let mut data: Vec<Vec<Cow<'a, str>>> = vec![vec![]; count];
        let blocks = item.as_slice();
        let styles = &self.styles;
        let mut any_filled = false;

        for idx in 0..count {
            for store in stores {
                if store.name() == blocks[idx] {
                    let hint = store.hint();
                    let help = store.help();

                    if !hint.is_empty() {
                        data[idx].push(hint);
                        any_filled = true;
                    }
                    if !help.is_empty() {
                        data[idx].push(help);
                        any_filled = true;
                    }
                }
            }
        }
        if !any_filled {
            return "".into();
        }
        let mut wrapper = Wrapper::new(&data);

        if !styles.is_empty() {
            wrapper.wrap_with(styles, self.max_width);
        } else {
            wrapper.wrap(self.max_width);
        }
        let wrapped = wrapper.get_output();
        let mut wrapped_lines = vec![];

        for wrapped_line in wrapped {
            let max_len = wrapped_line.iter().map(|v| v.len()).max().unwrap_or(1);
            let first_style = wrapped_line[0].get_style();
            let first_row_spacing = &" ".repeat(first_style.row_spacing);
            let first_line_spacing = &"\n".repeat(1 + first_style.line_spacing);

            for i in 0..max_len {
                let rows = &wrapped_line
                    .iter()
                    .map(|v| v.get_line(i))
                    .collect::<Vec<String>>();
                let mut line = rows.join(first_row_spacing);

                line.push_str(first_line_spacing);
                wrapped_lines.push(line);
            }
        }
        let wrapped_output = wrapped_lines.join("");

        if !wrapped_output.is_empty() {
            output.push(wrapped_output.into());
        }
        if !foot.is_empty() {
            output.push(foot);
        }
        let output = output.join(line_spacing);
        let output = output.trim_end().to_owned();

        output.into()
    }
}

impl<'a> HelpPolicy<'a, Command<'a>> for DefaultPolicy<'a, Command<'a>> {
    fn format(&self, item: &Command<'a>) -> Option<Cow<'a, str>> {
        let usage = self.get_command_usage(item);
        let mut blocks = vec![usage];
        let head = item.head();
        let foot = item.foot();
        let block_spacing = "\n".repeat(1 + self.style.block_spacing);

        if !head.is_empty() {
            blocks.push(head);
        }
        for block in item.block() {
            if !block.is_empty() {
                let help = self.get_block_help(block, item);

                if !help.is_empty() {
                    blocks.push(help);
                }
            }
        }
        if !foot.is_empty() {
            blocks.push(foot);
        }
        Some(blocks.join(&block_spacing).into())
    }
}

pub struct DefaultAppPolicy<'a, I> {
    styles: Vec<Style>, // style for every block

    max_width: usize,

    show_global: bool,

    hiding_pos: bool,

    usage_new_line: usize,

    marker: PhantomData<&'a I>,
}

impl<I> Default for DefaultAppPolicy<'_, I> {
    fn default() -> Self {
        Self {
            styles: Default::default(),
            max_width: 0,
            show_global: true,
            hiding_pos: true,
            usage_new_line: 0,
            marker: Default::default(),
        }
    }
}

impl<I> DefaultAppPolicy<'_, I> {
    pub fn new(
        styles: Vec<Style>,
        max_width: usize,
        show_global: bool,
        usage_new_line: usize,
    ) -> Self {
        Self {
            styles,
            max_width,
            show_global,
            hiding_pos: true,
            usage_new_line,
            marker: PhantomData,
        }
    }
}

impl<'a, W: Write> DefaultAppPolicy<'a, AppHelp<'a, W>> {
    pub fn get_block_usage(
        &self,
        item: &Block<'a, Cow<'a, str>>,
        stores: &[Store<'a>],
    ) -> (Vec<String>, Vec<String>) {
        let mut usages = vec![];
        let mut args = vec![];

        for store in item.iter() {
            if let Some(store) = stores.iter().find(|v| &v.name() == store) {
                let hint = store.hint();

                if !hint.is_empty() {
                    if store.position() {
                        if store.optional() {
                            args.push(format!("[{}]", hint));
                        } else {
                            args.push(format!("<{}>", hint));
                        }
                    } else if store.optional() {
                        usages.push(format!("[{}]", hint));
                    } else {
                        usages.push(format!("<{}>", hint));
                    }
                }
            }
        }
        (usages, args)
    }

    pub fn get_app_usage(&self, app: &AppHelp<'a, W>) -> Cow<'a, str> {
        let global = app.global();
        let mut usages = vec![];
        let mut args = vec![];
        let mut block_hint = vec![];

        for block in global.block() {
            let (block_usages, mut block_args) = self.get_block_usage(block, global);

            if !block_usages.is_empty() {
                for mut usage in block_usages {
                    if self.usage_new_line > 0 && (usages.len() + 1) % self.usage_new_line == 0 {
                        // add more space
                        // same length as `Usage: `
                        usage.push_str("\n      ");
                    }
                    usages.push(usage);
                }
            }
            // if not omit args, using the args, otherwise using hint of block
            if !block_args.is_empty() {
                args.append(&mut block_args);
            }
        }
        for block in global.block() {
            if !block.is_empty() {
                let arg = block.hint();

                if !arg.is_empty() {
                    block_hint.push(arg);
                }
            }
        }

        let mut ret = String::from("Usage: ");
        // all the option usage
        let global_usage = usages.join(" ");
        let block_hint = block_hint.join(" ");
        let command_usage = if app.has_cmd() { "<COMMAND>" } else { "" };
        let args = args.join(" ");

        if !global.name().is_empty() {
            ret += &global.name();
            ret += " ";
        }
        if !global_usage.is_empty() {
            ret += &global_usage;
            ret += " ";
        }
        if !command_usage.is_empty() {
            ret += command_usage;
            ret += " ";
        }
        if self.hiding_pos {
            if !block_hint.is_empty() {
                ret += &block_hint;
                ret += " ";
            }
        } else if !args.is_empty() {
            ret += &args;
            ret += " ";
        }
        ret.into()
    }

    pub fn get_block_help(
        &self,
        block: &Block<'a, Cow<'a, str>>,
        app: &AppHelp<'a, W>,
    ) -> Cow<'a, str> {
        let head = block.head();
        let foot = block.foot();
        let count = block.len();
        let mut data = vec![vec![]; count];
        let styles = &self.styles;
        let line_spacing = &"\n".repeat(1 + app.style().line_spacing);
        let mut any_filled = false;

        if block.is_empty() {
            return "".into();
        }
        for (name, data_mut) in block.iter().zip(data.iter_mut()) {
            if let Some(command) = app.find_cmd(name.clone()) {
                let hint = command.hint();
                let help = command.help();

                if !hint.is_empty() {
                    data_mut.push(hint);
                    any_filled = true;
                }
                if !help.is_empty() {
                    data_mut.push(help);
                    any_filled = true;
                }
            } else {
                panic!("Unknow command {} in block {}", name, block.name());
            }
        }
        if !any_filled {
            return "".into();
        }
        let mut wrapper = Wrapper::new(&data);

        if !styles.is_empty() {
            wrapper.wrap_with(styles, self.max_width);
        } else {
            wrapper.wrap(self.max_width);
        }
        let wrapped = wrapper.get_output();
        let mut wrapped_lines = vec![];

        for wrapped_line in wrapped {
            let max_len = wrapped_line.iter().map(|v| v.len()).max().unwrap_or(1);
            let first_style = wrapped_line[0].get_style();
            let first_row_spacing = &" ".repeat(first_style.row_spacing);
            let first_line_spacing = &"\n".repeat(1 + first_style.line_spacing);

            for i in 0..max_len {
                let rows = &wrapped_line
                    .iter()
                    .map(|v| v.get_line(i))
                    .collect::<Vec<String>>();
                let mut line = rows.join(first_row_spacing);

                line.push_str(first_line_spacing);
                wrapped_lines.push(line);
            }
        }

        if !wrapped_lines.is_empty() {
            // pop last line spacing
            wrapped_lines.last_mut().unwrap().pop();
        }

        let mut usages = vec![];
        let wrapped_output = wrapped_lines.join("");

        if !head.is_empty() {
            usages.push(head);
        }
        if !foot.is_empty() {
            usages.push(foot);
        }
        if !wrapped_output.is_empty() {
            usages.push(wrapped_output.into());
        }

        usages.join(line_spacing).into()
    }

    pub fn get_global_help(
        &self,
        item: &Block<'a, Cow<'a, str>>,
        stores: &Vec<Store<'a>>,
        app: &AppHelp<'a, W>,
    ) -> Cow<'a, str> {
        let style = &app.style;
        let count = item.len();
        let head = item.head();
        let foot = item.foot();
        let line_spacing = &"\n".repeat(1 + style.line_spacing);
        let mut output = if head.is_empty() { vec![] } else { vec![head] };
        let mut data: Vec<Vec<Cow<'a, str>>> = vec![vec![]; count];
        let blocks = item.as_slice();
        let styles = &self.styles;
        let mut any_filled = false;

        if item.is_empty() {
            return "".into();
        }
        for idx in 0..count {
            for store in stores {
                if store.name() == blocks[idx] {
                    let hint = store.hint();
                    let help = store.help();

                    if !hint.is_empty() {
                        data[idx].push(hint);
                        any_filled = true;
                    }
                    if !help.is_empty() {
                        data[idx].push(help);
                        any_filled = true;
                    }
                }
            }
        }
        if !any_filled {
            return "".into();
        }
        let mut wrapper = Wrapper::new(&data);

        if !styles.is_empty() {
            wrapper.wrap_with(styles, self.max_width);
        } else {
            wrapper.wrap(self.max_width);
        }
        let wrapped = wrapper.get_output();
        let mut wrapped_lines = vec![];

        for wrapped_line in wrapped {
            let max_len = wrapped_line.iter().map(|v| v.len()).max().unwrap_or(1);
            let first_style = wrapped_line[0].get_style();
            let first_row_spacing = &" ".repeat(first_style.row_spacing);
            let first_line_spacing = &"\n".repeat(1 + first_style.line_spacing);

            for i in 0..max_len {
                let rows = &wrapped_line
                    .iter()
                    .map(|v| v.get_line(i))
                    .collect::<Vec<String>>();
                let mut line = rows.join(first_row_spacing);

                line.push_str(first_line_spacing);
                wrapped_lines.push(line);
            }
        }
        let wrapped_output = wrapped_lines.join("");

        if !wrapped_output.is_empty() {
            output.push(wrapped_output.into());
        }
        if !foot.is_empty() {
            output.push(foot);
        }
        let output = output.join(line_spacing);
        let output = output.trim_end().to_owned();

        output.into()
    }
}

impl<'a, W: Write> HelpPolicy<'a, AppHelp<'a, W>> for DefaultAppPolicy<'a, AppHelp<'a, W>> {
    fn format(&self, app: &AppHelp<'a, W>) -> Option<Cow<'a, str>> {
        let usage = self.get_app_usage(app);
        let head = app.head();
        let foot = app.foot();
        let block_spacing = "\n".repeat(1 + app.style().block_spacing);
        let mut usages = if usage.is_empty() {
            vec![]
        } else {
            vec![usage]
        };

        if !head.is_empty() {
            usages.push(head);
        }
        for block in app.block().iter() {
            let block_help = self.get_block_help(block, app);

            if !block_help.is_empty() {
                usages.push(block_help);
            }
        }
        if self.show_global {
            for block in app.global().block() {
                let global_help = self.get_global_help(block, app.global(), app);

                if !global_help.is_empty() {
                    usages.push(global_help);
                }
            }
        }
        if !foot.is_empty() {
            usages.push(foot);
        }
        Some(usages.join(&block_spacing).into())
    }
}
