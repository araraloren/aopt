use aopt::opt::Opt;
use aopt::opt::Style;
use aopt::set::Set;
use aopt::Error;
use aopt_help::block::Block;
use aopt_help::store::Store;
use std::borrow::Cow;

pub const DEFAULT_OPTION_WIDTH: usize = 40;
pub const DEFAULT_USAGE_WIDTH: usize = 10;

#[derive(Debug, Clone, Default)]
pub struct HelpContext {
    name: String,

    head: String,

    foot: String,

    width: usize,

    usagew: usize,
}

impl HelpContext {
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    pub fn with_head(mut self, head: impl Into<String>) -> Self {
        self.head = head.into();
        self
    }

    pub fn with_foot(mut self, foot: impl Into<String>) -> Self {
        self.foot = foot.into();
        self
    }

    pub fn with_width(mut self, width: usize) -> Self {
        self.width = width;
        self
    }

    pub fn with_usagew(mut self, usagew: usize) -> Self {
        self.usagew = usagew;
        self
    }

    pub fn set_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = name.into();
        self
    }

    pub fn set_head(&mut self, head: impl Into<String>) -> &mut Self {
        self.head = head.into();
        self
    }

    pub fn set_foot(&mut self, foot: impl Into<String>) -> &mut Self {
        self.foot = foot.into();
        self
    }

    pub fn set_width(&mut self, width: usize) -> &mut Self {
        self.width = width;
        self
    }

    pub fn set_usagew(&mut self, usagew: usize) -> &mut Self {
        self.usagew = usagew;
        self
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn head(&self) -> &String {
        &self.head
    }

    pub fn foot(&self) -> &String {
        &self.foot
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn usagew(&self) -> usize {
        self.usagew
    }
}

pub fn display_set_help<'a, T: Set>(
    set: &T,
    name: impl Into<Cow<'a, str>>,
    head: impl Into<Cow<'a, str>>,
    foot: impl Into<Cow<'a, str>>,
    max_width: usize,
    usage_width: usize,
) -> Result<(), aopt_help::Error> {
    let mut app_help = aopt_help::AppHelp::new(
        name.into(),
        head.into(),
        foot.into(),
        aopt_help::prelude::Style::default(),
        std::io::stdout(),
        max_width,
        usage_width,
    );
    let global = app_help.global_mut();

    global.add_block(Block::new("command", "<COMMAND>", "", "Commands:", ""))?;
    global.add_block(Block::new("option", "", "", "Options:", ""))?;
    global.add_block(Block::new("args", "[ARGS]", "", "Args:", ""))?;
    for opt in set.iter() {
        if opt.mat_style(Style::Pos) {
            global.add_store(
                "args",
                Store::new(
                    Cow::from(opt.name()),
                    Cow::from(opt.hint()),
                    Cow::from(opt.help()),
                    Cow::default(),
                    !opt.force(),
                    true,
                ),
            )?;
        } else if opt.mat_style(Style::Cmd) {
            global.add_store(
                "command",
                Store::new(
                    Cow::from(opt.name()),
                    Cow::from(opt.hint()),
                    Cow::from(opt.help()),
                    Cow::default(),
                    !opt.force(),
                    true,
                ),
            )?;
        } else if opt.mat_style(Style::Argument)
            || opt.mat_style(Style::Boolean)
            || opt.mat_style(Style::Combined)
            || opt.mat_style(Style::Flag)
        {
            global.add_store(
                "option",
                Store::new(
                    Cow::from(opt.name()),
                    Cow::from(opt.hint()),
                    Cow::from(opt.help()),
                    Cow::default(),
                    !opt.force(),
                    false,
                ),
            )?;
        }
    }

    app_help.display(true)?;

    Ok(())
}

pub trait HelpDisplay<S: Set> {
    type Error: Into<Error>;

    fn display_if(
        &self,
        ctx: HelpContext,
        func: impl Fn(&Self) -> bool,
    ) -> Result<bool, Self::Error> {
        let ret = func(self);

        if ret {
            self.display(ctx)?;
        }
        Ok(ret)
    }

    fn display(&self, ctx: HelpContext) -> Result<(), Self::Error>;

    fn display_sub(&self, names: Vec<&str>, ctx: &HelpContext) -> Result<(), Self::Error>;
}
