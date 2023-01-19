use std::fmt::Debug;

use crate::opt::AOpt;
use crate::opt::Action;
use crate::opt::Cmd;
use crate::opt::ConfigValue;
use crate::opt::Main;
use crate::opt::Opt;
use crate::opt::OptConfig;
use crate::opt::Pos;
use crate::set::Ctor;
use crate::trace_log;
use crate::value::Infer;
use crate::value::RawValParser;
use crate::value::ValStorer;
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

#[derive(Debug, Clone, Copy)]
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
                Ok(AOpt::new(name, r#type, value)
                    .with_force(force)
                    .with_idx(index)
                    .with_action(action)
                    .with_alias(alias)
                    .with_style(styles)
                    .with_opt_help(help)
                    .with_ignore_name(ignore_name))
            },
        )
    }

    pub(crate) fn fill_infer_data<U: Infer>(info: &mut OptConfig)
    where
        U::Val: RawValParser,
    {
        let act = U::infer_act();
        let style = U::infer_style();
        let index = U::infer_index();
        let ignore_name = U::infer_ignore_name();
        let support_alias = U::infer_support_alias();
        let positional = U::infer_positional();
        let force = U::infer_force();
        let ctor = U::infer_ctor();

        (!info.has_ctor()).then(|| info.set_ctor(ctor));
        (!info.has_idx()).then(|| index.map(|idx| info.set_idx(idx)));
        (!info.has_type()).then(|| info.set_type::<U::Val>());
        (!info.has_action()).then(|| info.set_action(act));
        (!info.has_style()).then(|| info.set_style(style));
        (!info.has_force()).then(|| info.set_force(force));
        (!info.has_action()).then(|| info.set_action(act));
        if info.fix_infer() {
            if let Some(accessor) = info.take_accessor() {
                info.set_accessor(accessor.with_storer(ValStorer::new::<U::Val>()));
            }
        }
        info.set_ignore_name(ignore_name);
        info.set_support_alias(support_alias);
        info.set_postional(positional);
    }

    pub(crate) fn guess_default_infer(ctor: BuiltInCtor, info: &mut OptConfig) {
        match ctor {
            BuiltInCtor::Int => Self::fill_infer_data::<i64>(info),
            BuiltInCtor::Str => Self::fill_infer_data::<String>(info),
            BuiltInCtor::Flt => Self::fill_infer_data::<f64>(info),
            BuiltInCtor::Uint => Self::fill_infer_data::<u64>(info),
            BuiltInCtor::Bool => Self::fill_infer_data::<bool>(info),
            BuiltInCtor::Cmd => Self::fill_infer_data::<Cmd>(info),
            BuiltInCtor::Pos => Self::fill_infer_data::<Pos>(info),
            BuiltInCtor::Main => Self::fill_infer_data::<Main>(info),
            BuiltInCtor::Any => {}
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
