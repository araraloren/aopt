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

pub struct DefaultPolicy<'a, I> {
    name: Cow<'a, str>,

    style: Style,

    styles: Vec<Style>,

    omit_args: bool,

    marker: PhantomData<&'a I>,
}

impl<'a, I> Default for DefaultPolicy<'a, I> {
    fn default() -> Self {
        Self {
            name: Default::default(),
            style: Default::default(),
            styles: Default::default(),
            omit_args: true,
            marker: Default::default(),
        }
    }
}

impl<'a, I> DefaultPolicy<'a, I> {
    pub fn new<S: Into<Cow<'a, str>>>(
        name: S,
        style: Style,
        block: Vec<Style>,
        omit_args: bool,
    ) -> Self {
        Self {
            name: name.into(),
            style,
            styles: block,
            omit_args,
            marker: PhantomData::default(),
        }
    }
}

impl<'a> DefaultPolicy<'a, Command<'a>> {
    pub fn get_block_usage(
        &self,
        item: &Block<'a, Cow<'a, str>>,
        stores: &Vec<Store<'a>>,
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
                    } else {
                        if store.optional() {
                            usages.push(format!("[{}]", hint));
                        } else {
                            usages.push(format!("<{}>", hint));
                        }
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
            let (mut block_usages, mut block_args) = self.get_block_usage(block, &item);

            if !block_usages.is_empty() {
                usages.append(&mut block_usages);
            }
            // if not omit args, using the args, otherwise using hint of block
            if !self.omit_args && !block_args.is_empty() {
                args.append(&mut block_args);
            }
        }
        for block in item.block() {
            if self.omit_args {
                let arg = block.hint();

                if !arg.is_empty() {
                    block_hint.push(arg);
                }
            }
        }

        let usage = usages.join(" ");
        let args = args.join(" ");
        let block_hint = block_hint.join(" ");
        let ret;

        if self.omit_args {
            ret = format!(
                "Usage: {} {} {} {}",
                self.name,
                item.name(),
                usage,
                block_hint
            );
        } else {
            ret = format!("Usage: {} {} {} {}", self.name, item.name(), usage, args);
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

        for idx in 0..count {
            for store in stores {
                if store.name() == blocks[idx] {
                    let hint = store.hint();
                    let help = store.help();

                    if !hint.is_empty() {
                        data[idx].push(hint);
                    }
                    if !help.is_empty() {
                        data[idx].push(help);
                    }
                }
            }
        }
        let mut wrapper = Wrapper::new(&data);

        if !styles.is_empty() {
            wrapper.wrap_with(styles);
        } else {
            wrapper.wrap();
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
            let help = self.get_block_help(block, &item);

            if !help.is_empty() {
                blocks.push(help);
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

    show_global: bool,

    marker: PhantomData<&'a I>,
}

impl<'a, I> Default for DefaultAppPolicy<'a, I> {
    fn default() -> Self {
        Self {
            styles: Default::default(),
            show_global: true,
            marker: Default::default(),
        }
    }
}

impl<'a, I> DefaultAppPolicy<'a, I> {
    pub fn new(styles: Vec<Style>, show_global: bool) -> Self {
        Self {
            styles,
            show_global,
            marker: PhantomData::default(),
        }
    }
}

impl<'a, W: Write> DefaultAppPolicy<'a, AppHelp<'a, W>> {
    pub fn get_block_usage(&self, item: &Block<'a, Store<'a>>) -> Cow<'a, str> {
        let mut usages = vec![];

        for store in item.iter() {
            let hint = store.hint();

            if !hint.is_empty() {
                if store.optional() {
                    usages.push(format!("[{}]", hint));
                } else {
                    usages.push(format!("<{}>", hint));
                }
            }
        }
        usages.join(" ").into()
    }

    pub fn get_app_usage(&self, item: &AppHelp<'a, W>) -> Cow<'a, str> {
        let usage = self.get_block_usage(item.global());
        let command = if item.has_cmd() { " <COMMAND>" } else { "" };
        let usage_space = if usage.is_empty() { "" } else { " " };
        let args = if item.has_pos() { "[ARGS]" } else { "" };
        let ret = format!(
            "Usage: {}{usage_space}{}{} {}",
            item.name, usage, command, args
        );

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

        if block.is_empty() {
            return "".into();
        }
        for (name, data_mut) in block.iter().zip(data.iter_mut()) {
            if let Some(command) = app.find_cmd(name.clone()) {
                let hint = command.hint();
                let help = command.help();

                if !hint.is_empty() {
                    data_mut.push(hint);
                }
                if !help.is_empty() {
                    data_mut.push(help);
                }
            } else {
                panic!("Unknow command {} in block {}", name, block.name());
            }
        }
        let mut wrapper = Wrapper::new(&data);

        if !styles.is_empty() {
            wrapper.wrap_with(styles);
        } else {
            wrapper.wrap();
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

        usages.join(&line_spacing).into()
    }

    pub fn get_global_help(
        &self,
        item: &Block<'a, Store<'a>>,
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

        for idx in 0..count {
            let store = &blocks[idx];

            let hint = store.hint();
            let help = store.help();

            if !hint.is_empty() {
                data[idx].push(hint);
            }
            if !help.is_empty() {
                data[idx].push(help);
            }
        }
        let mut wrapper = Wrapper::new(&data);

        if !styles.is_empty() {
            wrapper.wrap_with(styles);
        } else {
            wrapper.wrap();
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
            let global_help = self.get_global_help(&app.global, app);

            if !global_help.is_empty() {
                usages.push(global_help);
            }
        }
        if !foot.is_empty() {
            usages.push(foot);
        }
        Some(usages.join(&block_spacing).into())
    }
}
