use std::ffi::OsString;
use std::fmt::Debug;

use crate::opt::AOpt;
use crate::opt::Action;
use crate::opt::Any;
use crate::opt::Cmd;
use crate::opt::ConfigValue;
use crate::opt::Main;
use crate::opt::Opt;
use crate::opt::OptConfig;
use crate::opt::Pos;
use crate::set::Ctor;
use crate::trace_log;
use crate::value::Infer;
use crate::value::ValAccessor;
use crate::Error;
use crate::Str;

#[cfg(feature = "sync")]
mod __creator {
    use super::*;

    pub struct Creator<O, C, E: Into<Error>> {
        pub(crate) name: Str,

        pub(crate) callback: Box<dyn FnMut(C) -> Result<O, E> + Send + Sync + 'static>,
    }

    impl<O: Opt, C, E: Into<Error>> Debug for Creator<O, C, E> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Creator")
                .field("name", &self.name)
                .field("callback", &"{...}")
                .finish()
        }
    }

    impl<O: Opt, C, E: Into<Error>> Creator<O, C, E> {
        pub fn new(
            name: Str,
            callback: impl FnMut(C) -> Result<O, E> + Send + Sync + 'static,
        ) -> Self {
            Self {
                name,
                callback: Box::new(callback),
            }
        }
    }
}

#[cfg(not(feature = "sync"))]
mod __creator {
    use super::*;

    pub struct Creator<O, C, E: Into<Error>> {
        pub(crate) name: Str,

        pub(crate) callback: Box<dyn FnMut(C) -> Result<O, E> + 'static>,
    }

    impl<O: Opt, C, E: Into<Error>> Debug for Creator<O, C, E> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Creator")
                .field("name", &self.name)
                .field("callback", &"{...}")
                .finish()
        }
    }

    impl<O: Opt, C, E: Into<Error>> Creator<O, C, E> {
        pub fn new(name: Str, callback: impl FnMut(C) -> Result<O, E> + 'static) -> Self {
            Self {
                name,
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

    fn name(&self) -> &Str {
        &self.name
    }

    fn new_with(&mut self, config: Self::Config) -> Result<Self::Opt, Self::Error> {
        (self.callback)(config)
    }
}

const BUILTIN_CTOR_INT: &str = "i";
const BUILTIN_CTOR_BOOL: &str = "b";
const BUILTIN_CTOR_UINT: &str = "u";
const BUILTIN_CTOR_STR: &str = "s";
const BUILTIN_CTOR_FLT: &str = "f";
const BUILTIN_CTOR_CMD: &str = "c";
const BUILTIN_CTOR_POS: &str = "p";
const BUILTIN_CTOR_MAIN: &str = "m";
const BUILTIN_CTOR_ANY: &str = "a";
const BUILTIN_CTOR_RAW: &str = "r";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BuiltInCtor {
    Int,

    Str,

    Flt,

    Uint,

    Bool,

    Cmd,

    Pos,

    Main,

    Any,

    Raw,
}

impl BuiltInCtor {
    pub fn name(&self) -> &str {
        match self {
            BuiltInCtor::Int => BUILTIN_CTOR_INT,
            BuiltInCtor::Str => BUILTIN_CTOR_STR,
            BuiltInCtor::Flt => BUILTIN_CTOR_FLT,
            BuiltInCtor::Uint => BUILTIN_CTOR_UINT,
            BuiltInCtor::Bool => BUILTIN_CTOR_BOOL,
            BuiltInCtor::Cmd => BUILTIN_CTOR_CMD,
            BuiltInCtor::Pos => BUILTIN_CTOR_POS,
            BuiltInCtor::Main => BUILTIN_CTOR_MAIN,
            BuiltInCtor::Any => BUILTIN_CTOR_ANY,
            BuiltInCtor::Raw => BUILTIN_CTOR_RAW,
        }
    }

    pub fn from_name(ctor: &Str) -> Self {
        match ctor.as_str() {
            BUILTIN_CTOR_INT => BuiltInCtor::Int,
            BUILTIN_CTOR_STR => BuiltInCtor::Str,
            BUILTIN_CTOR_FLT => BuiltInCtor::Flt,
            BUILTIN_CTOR_UINT => BuiltInCtor::Uint,
            BUILTIN_CTOR_BOOL => BuiltInCtor::Bool,
            BUILTIN_CTOR_CMD => BuiltInCtor::Cmd,
            BUILTIN_CTOR_POS => BuiltInCtor::Pos,
            BUILTIN_CTOR_MAIN => BuiltInCtor::Main,
            BUILTIN_CTOR_ANY => BuiltInCtor::Any,
            BUILTIN_CTOR_RAW => BuiltInCtor::Raw,
            name => {
                panic!("Unknow creator name: {}", name)
            }
        }
    }
}

impl Creator<AOpt, OptConfig, Error> {
    pub fn fallback() -> Self {
        Self::new(
            crate::set::ctor_default_name(),
            move |mut config: OptConfig| {
                trace_log!("Construct option with config {:?}", &config);

                let force = config.force().unwrap_or(false);
                let action = *config.action().unwrap_or(&Action::App);
                let storer = config.gen_storer()?;
                let initializer = config.gen_initializer()?;
                let ignore_name = config.ignore_name();
                let ignore_alias = config.ignore_alias();
                let ignore_index = config.ignore_index();
                let styles = config.gen_styles()?;
                let name = config.gen_name()?;
                let help = config.gen_opt_help()?;
                let r#type = config.gen_type()?;
                let index = config.index().cloned();
                let alias = config.take_alias();
                let alias = if alias.is_empty() { None } else { Some(alias) };

                if ignore_alias {
                    if let Some(alias) = &alias {
                        debug_assert!(
                            !alias.is_empty(),
                            "Option {} not support alias: {:?}",
                            name,
                            alias
                        );
                    }
                }
                if ignore_index {
                    if let Some(index) = &index {
                        debug_assert!(
                            !index.is_null(),
                            "Option {} not support position parameters: {:?}",
                            name,
                            index
                        );
                    }
                }
                Ok(
                    AOpt::new(name, r#type, ValAccessor::new(storer, initializer))
                        .with_force(force)
                        .with_idx(index)
                        .with_action(action)
                        .with_alias(alias)
                        .with_style(styles)
                        .with_opt_help(help)
                        .with_ignore_name(ignore_name)
                        .with_ignore_alias(ignore_alias)
                        .with_ignore_index(ignore_index),
                )
            },
        )
    }

    pub(crate) fn guess_default_infer(ctor: BuiltInCtor, info: &mut OptConfig) {
        match ctor {
            BuiltInCtor::Int => i64::infer_fill_info(info, false),
            BuiltInCtor::Str => String::infer_fill_info(info, false),
            BuiltInCtor::Flt => f64::infer_fill_info(info, false),
            BuiltInCtor::Uint => u64::infer_fill_info(info, false),
            BuiltInCtor::Bool => bool::infer_fill_info(info, false),
            BuiltInCtor::Cmd => Cmd::infer_fill_info(info, false),
            BuiltInCtor::Pos => Pos::<bool>::infer_fill_info(info, false),
            BuiltInCtor::Main => Main::<()>::infer_fill_info(info, false),
            BuiltInCtor::Any => Any::<()>::infer_fill_info(info, false),
            BuiltInCtor::Raw => OsString::infer_fill_info(info, false),
        }
    }

    pub fn new_type_ctor(ctor: BuiltInCtor) -> Self {
        let name = Str::from(ctor.name());

        Self::new(name, move |mut config: OptConfig| {
            trace_log!("Fill infer data for config {:?}", &config);

            Self::guess_default_infer(ctor, &mut config);

            trace_log!("Construct option with config {:?}", &config);

            let force = config.force().unwrap_or(false);
            let action = *config.action().unwrap_or(&Action::App);
            let storer = config.gen_storer()?;
            let initializer = config.gen_initializer()?;
            let ignore_name = config.ignore_name();
            let ignore_alias = config.ignore_alias();
            let ignore_index = config.ignore_index();
            let styles = config.gen_styles()?;
            let name = config.gen_name()?;
            let help = config.gen_opt_help()?;
            let r#type = config.gen_type()?;
            let index = config.index().cloned();
            let alias = config.take_alias();
            let alias = if alias.is_empty() { None } else { Some(alias) };

            if ignore_alias {
                if let Some(alias) = &alias {
                    debug_assert!(
                        !alias.is_empty(),
                        "Option {} not support alias: {:?}",
                        name,
                        alias
                    );
                }
            }
            if ignore_index {
                if let Some(index) = &index {
                    debug_assert!(
                        !index.is_null(),
                        "Option {} not support position parameters: {:?}",
                        name,
                        index
                    );
                }
            }
            Ok(
                AOpt::new(name, r#type, ValAccessor::new(storer, initializer))
                    .with_force(force)
                    .with_idx(index)
                    .with_action(action)
                    .with_alias(alias)
                    .with_style(styles)
                    .with_opt_help(help)
                    .with_ignore_name(ignore_name)
                    .with_ignore_alias(ignore_alias)
                    .with_ignore_index(ignore_index),
            )
        })
    }
}

impl From<BuiltInCtor> for Creator<AOpt, OptConfig, Error> {
    fn from(value: BuiltInCtor) -> Self {
        Self::new_type_ctor(value)
    }
}
