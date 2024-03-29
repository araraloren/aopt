use aopt::opt::Opt;
use aopt::opt::Style;
use aopt::set::Set;
use aopt_help::block::Block;
use aopt_help::store::Store;
use std::borrow::Cow;

#[derive(Debug, Clone, Default)]
pub struct HelpDisplayCtx {
    name: String,

    head: String,

    foot: String,

    width: usize,

    usagew: usize,

    subnames: Vec<String>,

    submode: bool,
}

impl HelpDisplayCtx {
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

    pub fn with_subnames(mut self, subnames: Vec<String>) -> Self {
        self.subnames = subnames;
        self
    }

    pub fn with_submode(mut self, submode: bool) -> Self {
        self.submode = submode;
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

    pub fn set_subnames(&mut self, subnames: Vec<String>) -> &mut Self {
        self.subnames = subnames;
        self
    }

    pub fn set_submode(&mut self, submode: bool) -> &mut Self {
        self.submode = submode;
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

    pub fn subnames(&self) -> &[String] {
        &self.subnames
    }

    pub fn submode(&self) -> bool {
        self.submode
    }

    pub fn generate_name(&self) -> String {
        if self.submode {
            std::iter::once(self.name())
                .chain(self.subnames().iter())
                .map(|v| v.as_str())
                .collect::<Vec<&str>>()
                .join(" ")
        } else {
            self.subnames()
                .iter()
                .chain(std::iter::once(self.name()))
                .map(|v| v.as_str())
                .collect::<Vec<&str>>()
                .join(" ")
        }
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
                    Cow::from(opt.name().as_str()),
                    Cow::from(opt.hint().as_str()),
                    Cow::from(opt.help().as_str()),
                    Cow::default(),
                    !opt.force(),
                    true,
                ),
            )?;
        } else if opt.mat_style(Style::Cmd) {
            global.add_store(
                "command",
                Store::new(
                    Cow::from(opt.name().as_str()),
                    Cow::from(opt.hint().as_str()),
                    Cow::from(opt.help().as_str()),
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
                    Cow::from(opt.name().as_str()),
                    Cow::from(opt.hint().as_str()),
                    Cow::from(opt.help().as_str()),
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

/// Using for cote-derive display help message.
#[macro_export]
macro_rules! display_help {
    ($set:ident, $name:expr, $head:expr, $foot:expr, $width:expr, $usage_width:expr) => {{
        fn __check_set<S: aopt::prelude::Set>(a: &S) -> &S {
            a
        }

        fn __check_name<T: Into<String>>(a: T) -> String {
            a.into()
        }

        $crate::display_set_help(
            __check_set($set),
            __check_name($name),
            $head,
            $foot,
            $width,
            $usage_width,
        )
        .map_err(|e| aopt::Error::raise_error(format!("Can not show help message: {:?}", e)))
    }};
    ($set:ident, $name:expr, $author:expr, $version:expr, $description:expr, $width:expr, $usage_width:expr) => {{
        let foot = format!("Create by {} v{}", $author, $version,);
        let head = format!("{}", $description);

        fn __check_set<S: aopt::prelude::Set>(a: &S) -> &S {
            a
        }

        fn __check_name<T: Into<String>>(a: T) -> String {
            a.into()
        }

        $crate::help::display_set_help(
            __check_set($set),
            __check_name($name),
            head,
            foot,
            $width,
            $usage_width,
        )
        .map_err(|e| aopt::Error::raise_error(format!("Can not show help message: {:?}", e)))
    }};
}
