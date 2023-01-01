use std::fmt::Debug;

use crate::astr;
use crate::opt::AOpt;
use crate::opt::Action;
use crate::opt::Assoc;
use crate::opt::ConfigValue;
use crate::opt::Opt;
use crate::opt::OptConfig;
use crate::opt::Style;
use crate::opt::ValInitiator;
use crate::opt::ValValidator;
use crate::set::Ctor;
use crate::Error;
use crate::Str;

cfg_if::cfg_if! {
    if #[cfg(feature = "sync")] {
        pub struct Creator<O, C, E: Into<Error>> {
            type_name: Str,

            callback: Box<dyn FnMut(C) -> Result<O, E> + Send + Sync + 'static>,
        }

        impl<O: Opt, C, E: Into<Error>> Debug for Creator<O, C, E> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct("Creator")
                    .field("type_name", &self.type_name)
                    .field("callback", &"{ ... }")
                    .finish()
            }
        }

        impl<O: Opt, C, E: Into<Error>> Creator<O, C, E> {
            pub fn new(
                type_name: Str,
                callback: impl FnMut(C) -> Result<O, E> + Send + Sync + 'static,
            ) -> Self {
                Self {
                    type_name,
                    callback: Box::new(callback),
                }
            }
        }
    }
    else {
        pub struct Creator<O, C, E: Into<Error>> {
            type_name: Str,

            callback: Box<dyn FnMut(C) -> Result<O, E> + 'static>,
        }

        impl<O: Opt, C, E: Into<Error>> Debug for Creator<O, C, E> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct("Creator")
                    .field("type_name", &self.type_name)
                    .field("callback", &"{ ... }")
                    .finish()
            }
        }

        impl<O: Opt, C, E: Into<Error>> Creator<O, C, E> {
            pub fn new(
                type_name: Str,
                callback: impl FnMut(C) -> Result<O, E> + 'static,
            ) -> Self {
                Self {
                    type_name,
                    callback: Box::new(callback),
                }
            }
        }
    }
}

impl<O: Opt, C, E: Into<Error>> Ctor for Creator<O, C, E> {
    type Opt = O;

    type Config = C;

    type Error = E;

    fn r#type(&self) -> Str {
        self.type_name.clone()
    }

    fn new_with(&mut self, config: Self::Config) -> Result<Self::Opt, Self::Error> {
        (self.callback)(config)
    }
}

impl Creator<AOpt, OptConfig, Error> {
    pub fn int() -> Self {
        let type_name = astr("i");

        Self::new(type_name.clone(), move |mut config: OptConfig| {
            let force = config.take_force().unwrap_or(false);
            let assoc = config.take_assoc().unwrap_or(Assoc::Int);
            let action = config.take_action().unwrap_or(Action::App);
            let initiator = config
                .take_initiator()
                .unwrap_or_else(ValInitiator::empty::<i64>);
            let validator = config
                .take_validator()
                .unwrap_or_else(|| ValValidator::i64());

            debug_assert!(
                config.idx().is_none(),
                "Int option not support index configruation"
            );
            if let Some(r#type) = config.r#type() {
                debug_assert_eq!(r#type, &type_name)
            }
            Ok(AOpt::default()
                .with_type(type_name.clone())
                .with_name(config.gen_name()?)
                .with_assoc(assoc)
                .with_action(action)
                .with_style(vec![Style::Argument])
                .with_opt_help(config.gen_opt_help()?)
                .with_alias(Some(config.gen_alias()?))
                .with_force(force)
                .with_initiator(initiator)
                .with_validator(validator))
        })
    }

    pub fn uint() -> Self {
        let type_name = astr("u");

        Self::new(type_name.clone(), move |mut config: OptConfig| {
            let force = config.take_force().unwrap_or(false);
            let assoc = config.take_assoc().unwrap_or(Assoc::Uint);
            let action = config.take_action().unwrap_or(Action::App);
            let initiator = config
                .take_initiator()
                .unwrap_or_else(ValInitiator::empty::<u64>);
            let validator = config
                .take_validator()
                .unwrap_or_else(|| ValValidator::u64());

            debug_assert!(
                config.idx().is_none(),
                "Uint option not support index configruation"
            );
            if let Some(r#type) = config.r#type() {
                debug_assert_eq!(r#type, &type_name)
            }
            Ok(AOpt::default()
                .with_type(type_name.clone())
                .with_name(config.gen_name()?)
                .with_assoc(assoc)
                .with_action(action)
                .with_style(vec![Style::Argument])
                .with_opt_help(config.gen_opt_help()?)
                .with_alias(Some(config.gen_alias()?))
                .with_force(force)
                .with_initiator(initiator)
                .with_validator(validator))
        })
    }

    pub fn flt() -> Self {
        let type_name = astr("f");

        Self::new(type_name.clone(), move |mut config: OptConfig| {
            let force = config.take_force().unwrap_or(false);
            let assoc = config.take_assoc().unwrap_or(Assoc::Flt);
            let action = config.take_action().unwrap_or(Action::App);
            let initiator = config
                .take_initiator()
                .unwrap_or_else(ValInitiator::empty::<f64>);
            let validator = config
                .take_validator()
                .unwrap_or_else(|| ValValidator::f64());

            debug_assert!(
                config.idx().is_none(),
                "Flt option not support index configruation"
            );
            if let Some(r#type) = config.r#type() {
                debug_assert_eq!(r#type, &type_name)
            }
            Ok(AOpt::default()
                .with_type(type_name.clone())
                .with_name(config.gen_name()?)
                .with_assoc(assoc)
                .with_action(action)
                .with_style(vec![Style::Argument])
                .with_opt_help(config.gen_opt_help()?)
                .with_alias(Some(config.gen_alias()?))
                .with_force(force)
                .with_initiator(initiator)
                .with_validator(validator))
        })
    }

    pub fn str() -> Self {
        let type_name = astr("s");

        Self::new(type_name.clone(), move |mut config: OptConfig| {
            let force = config.take_force().unwrap_or(false);
            let assoc = config.take_assoc().unwrap_or(Assoc::Str);
            let action = config.take_action().unwrap_or(Action::App);
            let initiator = config
                .take_initiator()
                .unwrap_or_else(ValInitiator::empty::<String>);
            let validator = config
                .take_validator()
                .unwrap_or_else(|| ValValidator::str());

            debug_assert!(
                config.idx().is_none(),
                "Str option not support index configruation"
            );
            if let Some(r#type) = config.r#type() {
                debug_assert_eq!(r#type, &type_name)
            }
            Ok(AOpt::default()
                .with_type(type_name.clone())
                .with_name(config.gen_name()?)
                .with_assoc(assoc)
                .with_action(action)
                .with_style(vec![Style::Argument])
                .with_opt_help(config.gen_opt_help()?)
                .with_alias(Some(config.gen_alias()?))
                .with_force(force)
                .with_initiator(initiator)
                .with_validator(validator))
        })
    }

    pub fn bool() -> Self {
        let type_name = astr("b");

        Self::new(type_name.clone(), move |mut config: OptConfig| {
            let force = config.take_force().unwrap_or(false);
            let assoc = config.take_assoc().unwrap_or(Assoc::Bool);
            let action = config.take_action().unwrap_or(Action::Set);
            let initiator = config
                .take_initiator()
                .unwrap_or_else(|| ValInitiator::bool(false));
            let validator = config
                .take_validator()
                .unwrap_or_else(|| ValValidator::bool());

            debug_assert!(
                config.idx().is_none(),
                "Boolean option not support index configruation"
            );
            if let Some(r#type) = config.r#type() {
                debug_assert_eq!(r#type, &type_name)
            }
            Ok(AOpt::default()
                .with_type(type_name.clone())
                .with_name(config.gen_name()?)
                .with_assoc(assoc)
                .with_action(action)
                .with_style(vec![Style::Boolean, Style::Combined])
                .with_opt_help(config.gen_opt_help()?)
                .with_alias(Some(config.gen_alias()?))
                .with_force(force)
                .with_initiator(initiator)
                .with_validator(validator))
        })
    }

    pub fn pos() -> Self {
        let type_name = astr("p");

        Self::new(type_name.clone(), move |mut config: OptConfig| {
            let force = config.take_force().unwrap_or(false);
            let assoc = config.take_assoc().unwrap_or(Assoc::Noa);
            let action = config.take_action().unwrap_or(Action::App);
            let initiator = config
                .take_initiator()
                .unwrap_or_else(ValInitiator::empty::<bool>);
            let validator = config.take_validator().unwrap_or_else(ValValidator::some);

            if let Some(v) = config.alias() {
                debug_assert!(v.is_empty(), "Pos option not support alias configruation")
            }
            if let Some(r#type) = config.r#type() {
                debug_assert_eq!(r#type, &type_name)
            }
            Ok(AOpt::default()
                .with_type(type_name.clone())
                .with_name(config.gen_name()?)
                .with_assoc(assoc)
                .with_action(action)
                .with_idx(Some(config.gen_idx()?))
                .with_style(vec![Style::Pos])
                .with_opt_help(config.gen_opt_help()?)
                .with_force(force)
                .with_initiator(initiator)
                .with_validator(validator)
                .with_ignore_name())
        })
    }

    pub fn cmd() -> Self {
        let type_name = astr("c");

        Self::new(type_name.clone(), move |mut config: OptConfig| {
            let assoc = config.take_assoc().unwrap_or(Assoc::Noa);
            let action = config.take_action().unwrap_or(Action::Set);
            let initiator = config
                .take_initiator()
                .unwrap_or_else(|| ValInitiator::bool(false));
            let validator = config.take_validator().unwrap_or_else(ValValidator::some);

            if let Some(v) = config.alias() {
                debug_assert!(v.is_empty(), "Cmd option not support alias configruation")
            }
            debug_assert!(
                config.force().unwrap_or(true),
                "Cmd option only have default optional configuration"
            );
            debug_assert!(
                config.idx().is_none(),
                "Cmd option only have default index configuration"
            );
            if let Some(r#type) = config.r#type() {
                debug_assert_eq!(r#type, &type_name)
            }
            Ok(AOpt::default()
                .with_type(type_name.clone())
                .with_name(config.gen_name()?)
                .with_assoc(assoc)
                .with_action(action)
                .with_idx(Some(crate::opt::Index::forward(1)))
                .with_style(vec![Style::Cmd])
                .with_opt_help(config.gen_opt_help()?)
                .with_force(true)
                .with_initiator(initiator)
                .with_validator(validator))
        })
    }

    pub fn main() -> Self {
        let type_name = astr("m");

        Self::new(type_name.clone(), move |mut config: OptConfig| {
            let assoc = config.take_assoc().unwrap_or(Assoc::Null);
            let action = config.take_action().unwrap_or(Action::Set);
            let initiator = config.take_initiator().unwrap_or_default();
            let validator = config.take_validator().unwrap_or_else(ValValidator::null);

            if let Some(v) = config.alias() {
                debug_assert!(v.is_empty(), "Main option not support alias configruation")
            }
            debug_assert!(
                !config.force().unwrap_or(false),
                "Main option only have default optional configuration"
            );
            debug_assert!(
                config.idx().is_none(),
                "Main option only have default index configuration"
            );
            if let Some(r#type) = config.r#type() {
                debug_assert_eq!(r#type, &type_name)
            }
            Ok(AOpt::default()
                .with_type(type_name.clone())
                .with_name(config.gen_name()?)
                .with_assoc(assoc)
                .with_action(action)
                .with_idx(Some(crate::opt::Index::anywhere()))
                .with_style(vec![Style::Main])
                .with_opt_help(config.gen_opt_help()?)
                .with_force(false)
                .with_initiator(initiator)
                .with_validator(validator)
                .with_ignore_name())
        })
    }

    pub fn any() -> Self {
        let type_name = astr("a");

        Self::new(type_name.clone(), move |mut config: OptConfig| {
            let assoc = config.take_assoc().unwrap_or(Assoc::Null);
            let action = config.take_action().unwrap_or(Action::Null);
            let initiator = config.take_initiator().unwrap_or_default();
            let validator = config.take_validator().unwrap_or_else(ValValidator::null);

            if let Some(r#type) = config.r#type() {
                debug_assert_eq!(r#type, &type_name)
            }
            Ok(AOpt::default()
                .with_type(type_name.clone())
                .with_name(config.gen_name()?)
                .with_assoc(assoc)
                .with_action(action)
                .with_idx(config.take_idx())
                .with_style(vec![
                    Style::Main,
                    Style::Pos,
                    Style::Cmd,
                    Style::Argument,
                    Style::Combined,
                    Style::Boolean,
                    Style::Null,
                ])
                .with_opt_help(config.gen_opt_help()?)
                .with_force(false)
                .with_initiator(initiator)
                .with_validator(validator))
        })
    }
}
