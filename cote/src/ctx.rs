use std::ops::{Deref, DerefMut};


#[derive(Debug, Clone,)]
pub struct FailedInfo {
    pub name: String,
    pub retval: aopt::prelude::ReturnVal,
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

#[derive(Debug, Clone, Default)]
pub struct RunningCtx {
    names: Vec<String>,

    display_help: bool,

    display_sub_help: bool,

    exit: bool,

    exit_sub: bool,

    failed_info: Vec<FailedInfo>,
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

    pub fn with_display_sub_help(mut self, display_sub_help: bool) -> Self {
        self.display_sub_help = display_sub_help;
        self
    }

    pub fn with_exit(mut self, exit: bool) -> Self {
        self.exit = exit;
        self
    }

    pub fn with_exit_sub(mut self, exit_sub: bool) -> Self {
        self.exit_sub = exit_sub;
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

    pub fn set_display_sub_help(&mut self, display_sub_help: bool) -> &mut Self {
        self.display_sub_help = display_sub_help;
        self
    }

    pub fn set_exit(&mut self, exit: bool) -> &mut Self {
        self.exit = exit;
        self
    }

    pub fn set_exit_sub(&mut self, exit_sub: bool) -> &mut Self {
        self.exit_sub = exit_sub;
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

    pub fn display_sub_help(&self) -> bool {
        self.display_sub_help
    }

    pub fn exit(&self) -> bool {
        self.exit
    }

    pub fn exit_sub(&self) -> bool {
        self.exit_sub
    }

    pub fn failed_info(&self) -> &[FailedInfo] {
        &self.failed_info
    }

    pub fn take_failed_info(&mut self) -> Vec<FailedInfo> {
        std::mem::take(&mut self.failed_info)
    }

    pub fn clear_failed_info(&mut self) {
        self.failed_info.clear();
    }

    pub fn add_name(&mut self, name: String) -> &mut Self {
        self.names.push(name);
        self
    }

    pub fn sync_ctx(&mut self, ctx: &mut Self) -> &mut Self {
        self.names.append(&mut ctx.names);
        self.display_help = self.display_help() || ctx.display_help();
        self.display_sub_help = self.display_sub_help() || ctx.display_sub_help();
        self.exit = self.exit() || ctx.exit();
        self.exit_sub = self.exit_sub() || ctx.exit_sub();
        self
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
