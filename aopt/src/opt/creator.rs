use std::fmt::Debug;

use crate::astr;
use crate::opt::AOpt;
use crate::opt::Action;
use crate::opt::ConfigValue;
use crate::opt::Opt;
use crate::opt::OptConfig;
use crate::prelude::Help;
use crate::raise_error;
use crate::set::Ctor;
use crate::trace_log;
use crate::value::ValAccessor;
use crate::AStr;
use crate::Error;

#[cfg(feature = "sync")]
mod __creator {
    use super::*;

    pub struct Creator<O, C, E: Into<Error>> {
        pub(crate) name: AStr,

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
            name: AStr,
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
        pub(crate) name: AStr,

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
        pub fn new(name: AStr, callback: impl FnMut(C) -> Result<O, E> + 'static) -> Self {
            Self {
                name,
                callback: Box::new(callback),
            }
        }
    }
}

pub use __creator::Creator;

use super::Index;

impl<O: Opt, C, E: Into<Error>> Ctor for Creator<O, C, E> {
    type Opt = O;

    type Config = C;

    type Error = E;

    fn name(&self) -> &AStr {
        &self.name
    }

    fn new_with(&mut self, config: Self::Config) -> Result<Self::Opt, Self::Error> {
        (self.callback)(config)
    }
}

const BUILTIN_CTOR_INT_SHORT: &str = "i";
const BUILTIN_CTOR_INT_LONG: &str = "int";
const BUILTIN_CTOR_INT_TYPE: &str = "i64";
const BUILTIN_CTOR_BOOL_SHORT: &str = "b";
const BUILTIN_CTOR_BOOL_LONG: &str = "boolean";
const BUILTIN_CTOR_BOOL_TYPE: &str = "bool";
const BUILTIN_CTOR_UINT_SHORT: &str = "u";
const BUILTIN_CTOR_UINT_LONG: &str = "uint";
const BUILTIN_CTOR_UINT_TYPE: &str = "u64";
const BUILTIN_CTOR_STR_SHORT: &str = "s";
const BUILTIN_CTOR_STR_LONG: &str = "str";
const BUILTIN_CTOR_STR_TYPE: &str = "string";
const BUILTIN_CTOR_FLT_SHORT: &str = "f";
const BUILTIN_CTOR_FLT_LONG: &str = "flt";
const BUILTIN_CTOR_FLT_TYPE: &str = "f64";
const BUILTIN_CTOR_CMD_SHORT: &str = "c";
const BUILTIN_CTOR_CMD_LONG: &str = "cmd";
const BUILTIN_CTOR_POS_SHORT: &str = "p";
const BUILTIN_CTOR_POS_LONG: &str = "pos";
const BUILTIN_CTOR_MAIN_SHORT: &str = "m";
const BUILTIN_CTOR_MAIN_LONG: &str = "main";
const BUILTIN_CTOR_ANY_SHORT: &str = "a";
const BUILTIN_CTOR_ANY_LONG: &str = "any";
const BUILTIN_CTOR_RAW_SHORT: &str = "r";
const BUILTIN_CTOR_RAW_LONG: &str = "raw";
const BUILTIN_CTOR_FALLBACK: &str = "fallback";

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BuiltInCtor {
    /// Create names: `i`, `int`, `i64`
    Int,

    /// Create names: `s`, `str`, `string`
    AStr,

    /// Create names: `f`, `flt`, `f64`
    Flt,

    /// Create names: `u`, `uint`, `u64`
    Uint,

    /// Create names: `b`, `boolean`, `bool`
    Bool,

    /// Create names: `c`, `cmd`
    Cmd,

    /// Create names: `p`, `pos`
    Pos,

    /// Create names: `m`, `main`
    Main,

    /// Create names: `a`, `any`
    Any,

    /// Create names: `r`, `raw`
    Raw,

    /// Create names: `fallback`
    Fallback,
}

impl BuiltInCtor {
    pub fn name(&self) -> &str {
        match self {
            BuiltInCtor::Int => BUILTIN_CTOR_INT_SHORT,
            BuiltInCtor::AStr => BUILTIN_CTOR_STR_SHORT,
            BuiltInCtor::Flt => BUILTIN_CTOR_FLT_SHORT,
            BuiltInCtor::Uint => BUILTIN_CTOR_UINT_SHORT,
            BuiltInCtor::Bool => BUILTIN_CTOR_BOOL_SHORT,
            BuiltInCtor::Cmd => BUILTIN_CTOR_CMD_SHORT,
            BuiltInCtor::Pos => BUILTIN_CTOR_POS_SHORT,
            BuiltInCtor::Main => BUILTIN_CTOR_MAIN_SHORT,
            BuiltInCtor::Any => BUILTIN_CTOR_ANY_SHORT,
            BuiltInCtor::Raw => BUILTIN_CTOR_RAW_SHORT,
            BuiltInCtor::Fallback => BUILTIN_CTOR_FALLBACK,
        }
    }

    pub fn from_name<S: AsRef<str>>(ctor: &S) -> Self {
        match ctor.as_ref() {
            BUILTIN_CTOR_INT_SHORT | BUILTIN_CTOR_INT_LONG | BUILTIN_CTOR_INT_TYPE => {
                BuiltInCtor::Int
            }
            BUILTIN_CTOR_STR_SHORT | BUILTIN_CTOR_STR_LONG | BUILTIN_CTOR_STR_TYPE => {
                BuiltInCtor::AStr
            }
            BUILTIN_CTOR_FLT_SHORT | BUILTIN_CTOR_FLT_LONG | BUILTIN_CTOR_FLT_TYPE => {
                BuiltInCtor::Flt
            }
            BUILTIN_CTOR_UINT_SHORT | BUILTIN_CTOR_UINT_LONG | BUILTIN_CTOR_UINT_TYPE => {
                BuiltInCtor::Uint
            }
            BUILTIN_CTOR_BOOL_SHORT | BUILTIN_CTOR_BOOL_LONG | BUILTIN_CTOR_BOOL_TYPE => {
                BuiltInCtor::Bool
            }
            BUILTIN_CTOR_CMD_SHORT | BUILTIN_CTOR_CMD_LONG => BuiltInCtor::Cmd,
            BUILTIN_CTOR_POS_SHORT | BUILTIN_CTOR_POS_LONG => BuiltInCtor::Pos,
            BUILTIN_CTOR_MAIN_SHORT | BUILTIN_CTOR_MAIN_LONG => BuiltInCtor::Main,
            BUILTIN_CTOR_ANY_SHORT | BUILTIN_CTOR_ANY_LONG => BuiltInCtor::Any,
            BUILTIN_CTOR_RAW_SHORT | BUILTIN_CTOR_RAW_LONG => BuiltInCtor::Raw,
            BUILTIN_CTOR_FALLBACK => BuiltInCtor::Fallback,
            name => {
                panic!("Unknow creator name: {}", name)
            }
        }
    }
}

fn gen_hint(hint: Option<&AStr>, n: &AStr, idx: Option<&Index>, alias: Option<&Vec<AStr>>) -> AStr {
    let hint_generator = || {
        let mut names = Vec::with_capacity(1 + alias.map(|v| v.len()).unwrap_or_default());

        // add name
        names.push(n.as_str());
        // add alias
        if let Some(alias_vec) = alias {
            for alias in alias_vec {
                names.push(alias.as_str());
            }
        }
        // sort name by len
        names.sort_by_key(|v| v.len());
        astr(if let Some(index) = idx {
            let index_string = index.to_help();

            // add index string
            if index_string.is_empty() {
                names.join(", ")
            } else {
                format!("{}@{}", names.join(", "), index_string)
            }
        } else {
            names.join(", ")
        })
    };

    hint.cloned().unwrap_or_else(hint_generator)
}

impl Creator<AOpt, OptConfig, Error> {
    pub fn fallback() -> Self {
        Self::new(
            crate::set::ctor_default_name(),
            move |mut config: OptConfig| {
                trace_log!("Construct option with config {:?}", &config);

                let r#type = config.take_type();
                let name = config.take_name();
                let force = config.take_force();
                let index = config.take_index();
                let alias = config.take_alias();
                let hint = config.take_hint();
                let help = config.take_help();
                let action = config.take_action();
                let storer = config.take_storer();
                let styles = config.take_style();
                let initializer = config.take_initializer();
                let ignore_name = config.ignore_name();
                let ignore_alias = config.ignore_alias();
                let ignore_index = config.ignore_index();

                let force = force.unwrap_or(false);
                let action = action.unwrap_or(Action::App);
                let storer = storer.ok_or_else(|| {
                    raise_error!("Incomplete option configuration: missing ValStorer")
                })?;
                let initializer = initializer.ok_or_else(|| {
                    raise_error!("Incomplete option configuration: missing ValInitializer")
                })?;
                let styles = styles.ok_or_else(|| {
                    raise_error!("Incomplete option configuration: missing Style")
                })?;
                let name = name.ok_or_else(|| {
                    raise_error!("Incomplete option configuration: missing option name")
                })?;
                let hint = gen_hint(hint.as_ref(), &name, index.as_ref(), alias.as_ref());
                let help = help.unwrap_or_default();
                let r#type = r#type.ok_or_else(|| {
                    raise_error!("Incomplete option configuration: missing option value type")
                })?;
                let help = Help::default().with_help(help).with_hint(hint);

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
                            "Please remove the index, option `{}` not support positional parameters: {:?}",
                            name,
                            index
                        );
                    }
                } else {
                    debug_assert!(
                        index.is_some(),
                        "Please provide an index, indicate the position you want to capture for option `{}`.",
                        name
                    );
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

    pub fn new_type_ctor(ctor: BuiltInCtor) -> Self {
        if ctor == BuiltInCtor::Fallback {
            return Self::fallback();
        }
        let name = AStr::from(ctor.name());

        Self::new(name, move |mut config: OptConfig| {
            trace_log!("Construct option with config {:?}", &config);

            let r#type = config.take_type();
            let name = config.take_name();
            let force = config.take_force();
            let index = config.take_index();
            let alias = config.take_alias();
            let hint = config.take_hint();
            let help = config.take_help();
            let action = config.take_action();
            let storer = config.take_storer();
            let styles = config.take_style();
            let initializer = config.take_initializer();
            let ignore_name = config.ignore_name();
            let ignore_alias = config.ignore_alias();
            let ignore_index = config.ignore_index();

            let force = force.unwrap_or(false);
            let action = action.unwrap_or(Action::App);
            let storer = storer.ok_or_else(|| {
                raise_error!("Incomplete option configuration: missing ValStorer")
            })?;
            let initializer = initializer.ok_or_else(|| {
                raise_error!("Incomplete option configuration: missing ValInitializer")
            })?;
            let styles = styles
                .ok_or_else(|| raise_error!("Incomplete option configuration: missing Style"))?;
            let name = name.ok_or_else(|| {
                raise_error!("Incomplete option configuration: missing option name")
            })?;
            let hint = gen_hint(hint.as_ref(), &name, index.as_ref(), alias.as_ref());
            let help = help.unwrap_or_default();
            let r#type = r#type.ok_or_else(|| {
                raise_error!("Incomplete option configuration: missing option value type")
            })?;
            let help = Help::default().with_help(help).with_hint(hint);

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
                        "Please remove the index, option `{}` not support positional parameters: {:?}",
                        name,
                        index
                    );
                }
            } else {
                debug_assert!(
                    index.is_some(),
                    "Please provide an index, indicate the position you want to capture for option `{}`.",
                    name
                );
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

/// Return an array of creators:
///
/// * [`Fallback`](BuiltInCtor::Fallback)
/// * [`Int`](BuiltInCtor::Int)
/// * [`Bool`](BuiltInCtor::Bool)
/// * [`Flt`](BuiltInCtor::Flt)
/// * [`AStr`](BuiltInCtor::AStr)
/// * [`Uint`](BuiltInCtor::Uint)
/// * [`Cmd`](BuiltInCtor::Cmd)
/// * [`Pos`](BuiltInCtor::Pos)
/// * [`Main`](BuiltInCtor::Main)
/// * [`Any`](BuiltInCtor::Any)
/// * [`Raw`](BuiltInCtor::Raw)
#[macro_export]
macro_rules! ctors {
    ($type:ident) => {
        $crate::ctors!(
            $type,
            fallback,
            int,
            bool,
            flt,
            str,
            uint,
            cmd,
            pos,
            main,
            any,
            raw
        )
    };
    ($type:ident, $($creator:ident),+) => {
        {
            vec![
                $(
                    <$type>::from(
                        $crate::opt::BuiltInCtor::from_name(
                            &stringify!($creator)
                    )),
                )+
            ]
        }
    };
}
