use crate::prelude::HelpContext;
use crate::ReturnVal;
use std::ops::Deref;
use std::ops::DerefMut;

#[derive(Debug, Clone)]
pub struct FailedInfo {
    pub name: String,
    pub retval: ReturnVal,
}

impl FailedInfo {
    pub fn new(name: String, retval: ReturnVal) -> Self {
        Self { name, retval }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn retval(&self) -> &ReturnVal {
        &self.retval
    }
}

impl Deref for FailedInfo {
    type Target = aopt::prelude::ReturnVal;

    fn deref(&self) -> &Self::Target {
        &self.retval
    }
}

impl DerefMut for FailedInfo {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.retval
    }
}

/// Collect running information when do parsing.
#[derive(Debug, Clone, Default)]
pub struct RunningCtx {
    names: Vec<String>,

    display_help: bool,

    sub_parser: bool,

    exit: bool,

    failed_info: Vec<FailedInfo>,

    help_context: Option<HelpContext>,
}

impl RunningCtx {
    pub fn with_names(mut self, names: Vec<String>) -> Self {
        self.names = names;
        self
    }

    pub fn with_display_help(mut self, display_help: bool) -> Self {
        self.display_help = display_help;
        self
    }

    pub fn with_sub_parser(mut self, sub_parser: bool) -> Self {
        self.sub_parser = sub_parser;
        self
    }

    pub fn with_exit(mut self, exit: bool) -> Self {
        self.exit = exit;
        self
    }

    pub fn with_help_context(mut self, help_context: HelpContext) -> Self {
        self.help_context = Some(help_context);
        self
    }

    pub fn set_names(&mut self, names: Vec<String>) -> &mut Self {
        self.names = names;
        self
    }

    pub fn set_display_help(&mut self, display_help: bool) -> &mut Self {
        self.display_help = display_help;
        self
    }

    pub fn set_sub_parser(&mut self, sub_parser: bool) -> &mut Self {
        self.sub_parser = sub_parser;
        self
    }

    pub fn set_exit(&mut self, exit: bool) -> &mut Self {
        self.exit = exit;
        self
    }

    pub fn set_help_context(&mut self, help_context: HelpContext) -> &mut Self {
        self.help_context = Some(help_context);
        self
    }

    pub fn add_failed_info(&mut self, failed_info: FailedInfo) -> &mut Self {
        self.failed_info.push(failed_info);
        self
    }

    pub fn names(&self) -> &[String] {
        &self.names
    }

    pub fn display_help(&self) -> bool {
        self.display_help
    }

    pub fn sub_parser(&self) -> bool {
        self.sub_parser
    }

    pub fn exit(&self) -> bool {
        self.exit
    }

    pub fn failed_info(&self) -> &[FailedInfo] {
        &self.failed_info
    }

    pub fn help_context(&self) -> Option<&HelpContext> {
        self.help_context.as_ref()
    }

    pub fn take_failed_info(&mut self) -> Vec<FailedInfo> {
        std::mem::take(&mut self.failed_info)
    }

    pub fn take_help_context(&mut self) -> Option<HelpContext> {
        self.help_context.take()
    }

    pub fn clear_failed_info(&mut self) {
        self.failed_info.clear();
    }

    pub fn add_name(&mut self, name: String) -> &mut Self {
        self.names.push(name);
        self
    }

    pub fn pop_name(&mut self) -> Option<String> {
        self.names.pop()
    }

    pub fn sync_failed_info(&mut self, ctx: &mut Self) -> &mut Self {
        self.failed_info.extend(ctx.take_failed_info());
        self
    }

    pub fn chain_error(&mut self) -> Option<aopt::Error> {
        let mut iter = self.failed_info.iter_mut();

        if let Some(failed_info) = iter.next() {
            let mut error = failed_info.take_failure();

            for failed_info in iter {
                error = error.cause(failed_info.take_failure());
            }
            Some(error)
        } else {
            None
        }
    }
}
