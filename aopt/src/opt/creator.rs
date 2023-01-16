use std::fmt::Debug;

use crate::opt::AOpt;
use crate::opt::Action;
use crate::opt::ConfigValue;
use crate::opt::Opt;
use crate::opt::OptConfig;
use crate::set::Ctor;
use crate::Error;

#[cfg(feature = "sync")]
mod __creator {
    use super::*;

    pub struct Creator<O, C, E: Into<Error>> {
        pub(crate) callback: Box<dyn FnMut(C) -> Result<O, E> + Send + Sync + 'static>,
    }

    impl<O: Opt, C, E: Into<Error>> Debug for Creator<O, C, E> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Creator")
                .field("callback", &"{ ... }")
                .finish()
        }
    }

    impl<O: Opt, C, E: Into<Error>> Creator<O, C, E> {
        pub fn new(callback: impl FnMut(C) -> Result<O, E> + Send + Sync + 'static) -> Self {
            Self {
                callback: Box::new(callback),
            }
        }
    }
}

#[cfg(not(feature = "sync"))]
mod __creator {
    use super::*;

    pub struct Creator<O, C, E: Into<Error>> {
        pub(crate) callback: Box<dyn FnMut(C) -> Result<O, E> + 'static>,
    }

    impl<O: Opt, C, E: Into<Error>> Debug for Creator<O, C, E> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Creator")
                .field("callback", &"{ ... }")
                .finish()
        }
    }

    impl<O: Opt, C, E: Into<Error>> Creator<O, C, E> {
        pub fn new(callback: impl FnMut(C) -> Result<O, E> + 'static) -> Self {
            Self {
                callback: Box::new(callback),
            }
        }
    }
}

pub use __creator::Creator;

impl<O: Opt, C, E: Into<Error>> Ctor for Creator<O, C, E> {
    type Opt = O;

    type Config = C;

    type Error = E;

    fn new_with(&mut self, config: Self::Config) -> Result<Self::Opt, Self::Error> {
        (self.callback)(config)
    }
}

impl Creator<AOpt, OptConfig, Error> {
    pub fn fallback() -> Self {
        Self::new(move |mut config: OptConfig| {
            let force = config.force().unwrap_or(false);
            let action = *config.action().unwrap_or(&Action::App);
            let value = config.gen_accessor()?;
            let ignore_name = config.ignore_name();
            let support_alias = config.support_alias();
            let positional = config.positional();
            let styles = config.gen_styles()?;
            let name = config.gen_name()?;
            let help = config.gen_opt_help()?;
            let r#type = config.gen_type()?;
            let index = config.idx().cloned();
            let alias = config.take_alias();
            let alias = if alias.is_empty() { None } else { Some(alias) };

            if !support_alias {
                if let Some(alias) = &alias {
                    debug_assert!(
                        !alias.is_empty(),
                        "Option {} not support alias: {:?}",
                        name,
                        alias
                    );
                }
            }
            if !positional {
                if let Some(index) = &index {
                    debug_assert!(
                        !index.is_null(),
                        "Option {} not support position parameters: {:?}",
                        name,
                        index
                    );
                }
            }
            // do some check ?
            Ok(AOpt::new(name, r#type, value)
                .with_force(force)
                .with_idx(index)
                .with_action(action)
                .with_alias(alias)
                .with_style(styles)
                .with_opt_help(help)
                .with_ignore_name(ignore_name))
        })
    }
}
