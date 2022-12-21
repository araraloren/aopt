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

            deactivate: bool,

            callback: Box<dyn FnMut(C) -> Result<O, E> + Send + Sync + 'static>,
        }

        impl<O: Opt, C, E: Into<Error>> Debug for Creator<O, C, E> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct("Creator")
                    .field("type_name", &self.type_name)
                    .field("deactivate", &self.deactivate)
                    .field("callback", &"{ ... }")
                    .finish()
            }
        }

        impl<O: Opt, C, E: Into<Error>> Creator<O, C, E> {
            pub fn new(
                type_name: Str,
                deactivate: bool,
                callback: impl FnMut(C) -> Result<O, E> + Send + Sync + 'static,
            ) -> Self {
                Self {
                    type_name,
                    deactivate,
                    callback: Box::new(callback),
                }
            }
        }
    }
    else {
        pub struct Creator<O, C, E: Into<Error>> {
            type_name: Str,

            deactivate: bool,

            callback: Box<dyn FnMut(C) -> Result<O, E> + 'static>,
        }

        impl<O: Opt, C, E: Into<Error>> Debug for Creator<O, C, E> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct("Creator")
                    .field("type_name", &self.type_name)
                    .field("deactivate", &self.deactivate)
                    .field("callback", &"{ ... }")
                    .finish()
            }
        }

        impl<O: Opt, C, E: Into<Error>> Creator<O, C, E> {
            pub fn new(
                type_name: Str,
                deactivate: bool,
                callback: impl FnMut(C) -> Result<O, E> + 'static,
            ) -> Self {
                Self {
                    type_name,
                    deactivate,
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

    fn sp_deactivate(&self) -> bool {
        self.deactivate
    }

    fn new_with(&mut self, config: Self::Config) -> Result<Self::Opt, Self::Error> {
        (self.callback)(config)
    }
}

impl Creator<AOpt, OptConfig, Error> {
    pub fn int() -> Self {
        let type_name = astr("i");

        Self::new(type_name.clone(), false, move |mut config: OptConfig| {
            let deactivate_style = config.deactivate().unwrap_or(false);
            let prefix = Some(config.gen_prefix()?);
            let optional = config.take_optional().unwrap_or(true);
            let assoc = config.take_assoc().unwrap_or(Assoc::Int);
            let action = config.take_action().unwrap_or(Action::App);
            let initiator = config
                .take_initiator()
                .unwrap_or_else(ValInitiator::empty::<i64>);

            debug_assert_eq!(
                assoc,
                Assoc::Int,
                "The type must be ValType::Int for Int option"
            );
            debug_assert!(
                config.idx().is_none(),
                "Int option not support index configruation"
            );
            debug_assert!(
                !config.has_validator(),
                "Int option only have default value validator"
            );
            debug_assert!(
                !deactivate_style,
                "Int option not support deactivate style configuration"
            );
            if let Some(r#type) = config.r#type() {
                debug_assert_eq!(r#type, &type_name)
            }
            Ok(AOpt::default()
                .with_type(type_name.clone())
                .with_name(config.gen_name()?)
                .with_prefix(prefix)
                .with_assoc(assoc)
                .with_action(action)
                .with_style(vec![Style::Argument])
                .with_opt_help(config.gen_opt_help(false)?)
                .with_alias(Some(config.gen_alias()?))
                .with_optional(optional)
                .with_initiator(initiator)
                .with_validator(ValValidator::i64()))
        })
    }

    pub fn uint() -> Self {
        let type_name = astr("u");

        Self::new(type_name.clone(), false, move |mut config: OptConfig| {
            let deactivate_style = config.deactivate().unwrap_or(false);
            let prefix = Some(config.gen_prefix()?);
            let optional = config.take_optional().unwrap_or(true);
            let assoc = config.take_assoc().unwrap_or(Assoc::Uint);
            let action = config.take_action().unwrap_or(Action::App);
            let initiator = config
                .take_initiator()
                .unwrap_or_else(ValInitiator::empty::<u64>);

            debug_assert_eq!(
                assoc,
                Assoc::Uint,
                "The type must be ValType::Uint for Uint option"
            );
            debug_assert!(
                config.idx().is_none(),
                "Uint option not support index configruation"
            );
            debug_assert!(
                !config.has_validator(),
                "Uint option only have default value validator"
            );
            debug_assert!(
                !deactivate_style,
                "Uint option not support deactivate style configuration"
            );
            if let Some(r#type) = config.r#type() {
                debug_assert_eq!(r#type, &type_name)
            }
            Ok(AOpt::default()
                .with_type(type_name.clone())
                .with_name(config.gen_name()?)
                .with_prefix(prefix)
                .with_assoc(assoc)
                .with_action(action)
                .with_style(vec![Style::Argument])
                .with_opt_help(config.gen_opt_help(false)?)
                .with_alias(Some(config.gen_alias()?))
                .with_optional(optional)
                .with_initiator(initiator)
                .with_validator(ValValidator::u64()))
        })
    }

    pub fn flt() -> Self {
        let type_name = astr("f");

        Self::new(type_name.clone(), false, move |mut config: OptConfig| {
            let deactivate_style = config.deactivate().unwrap_or(false);
            let prefix = Some(config.gen_prefix()?);
            let optional = config.take_optional().unwrap_or(true);
            let assoc = config.take_assoc().unwrap_or(Assoc::Flt);
            let action = config.take_action().unwrap_or(Action::App);
            let initiator = config
                .take_initiator()
                .unwrap_or_else(ValInitiator::empty::<f64>);

            debug_assert_eq!(
                assoc,
                Assoc::Flt,
                "The type must be ValType::Flt for Flt option"
            );
            debug_assert!(
                config.idx().is_none(),
                "Flt option not support index configruation"
            );
            debug_assert!(
                !config.has_validator(),
                "Flt option only have default value validator"
            );
            debug_assert!(
                !deactivate_style,
                "Flt option not support deactivate style configuration"
            );
            if let Some(r#type) = config.r#type() {
                debug_assert_eq!(r#type, &type_name)
            }
            Ok(AOpt::default()
                .with_type(type_name.clone())
                .with_name(config.gen_name()?)
                .with_prefix(prefix)
                .with_assoc(assoc)
                .with_action(action)
                .with_style(vec![Style::Argument])
                .with_opt_help(config.gen_opt_help(false)?)
                .with_alias(Some(config.gen_alias()?))
                .with_optional(optional)
                .with_initiator(initiator)
                .with_validator(ValValidator::f64()))
        })
    }

    pub fn str() -> Self {
        let type_name = astr("s");

        Self::new(type_name.clone(), false, move |mut config: OptConfig| {
            let deactivate_style = config.deactivate().unwrap_or(false);
            let prefix = Some(config.gen_prefix()?);
            let optional = config.take_optional().unwrap_or(true);
            let assoc = config.take_assoc().unwrap_or(Assoc::Str);
            let action = config.take_action().unwrap_or(Action::App);
            let initiator = config
                .take_initiator()
                .unwrap_or_else(ValInitiator::empty::<String>);

            debug_assert_eq!(
                assoc,
                Assoc::Str,
                "The type must be ValType::Str for Str option"
            );
            debug_assert!(
                config.idx().is_none(),
                "Str option not support index configruation"
            );
            debug_assert!(
                !config.has_validator(),
                "Str option only have default value validator"
            );
            debug_assert!(
                !deactivate_style,
                "Str option not support deactivate style configuration"
            );
            if let Some(r#type) = config.r#type() {
                debug_assert_eq!(r#type, &type_name)
            }
            Ok(AOpt::default()
                .with_type(type_name.clone())
                .with_name(config.gen_name()?)
                .with_prefix(prefix)
                .with_assoc(assoc)
                .with_action(action)
                .with_style(vec![Style::Argument])
                .with_opt_help(config.gen_opt_help(false)?)
                .with_alias(Some(config.gen_alias()?))
                .with_optional(optional)
                .with_initiator(initiator)
                .with_validator(ValValidator::str()))
        })
    }

    pub fn bool() -> Self {
        let type_name = astr("b");

        Self::new(type_name.clone(), true, move |mut config: OptConfig| {
            let deactivate_style = config.deactivate().unwrap_or(false);
            let prefix = Some(config.gen_prefix()?);
            let optional = config.take_optional().unwrap_or(true);
            let assoc = config.take_assoc().unwrap_or(Assoc::Bool);
            let action = config.take_action().unwrap_or(Action::Set);
            let value = deactivate_style;

            debug_assert_eq!(
                assoc,
                Assoc::Bool,
                "The type must be ValType::Bool for Boolean option"
            );
            debug_assert!(
                config.idx().is_none(),
                "Boolean option not support index configruation"
            );
            debug_assert!(
                !config.has_validator(),
                "Boolean option only have default value validator"
            );
            debug_assert!(
                !config.has_initiator(),
                "Boolean option only have default value initiator"
            );
            if let Some(r#type) = config.r#type() {
                debug_assert_eq!(r#type, &type_name)
            }
            Ok(AOpt::default()
                .with_type(type_name.clone())
                .with_name(config.gen_name()?)
                .with_prefix(prefix)
                .with_assoc(assoc)
                .with_action(action)
                .with_style(vec![Style::Boolean, Style::Combined])
                .with_opt_help(config.gen_opt_help(deactivate_style)?)
                .with_alias(Some(config.gen_alias()?))
                .with_optional(optional)
                .with_initiator(ValInitiator::bool(value))
                .with_validator(ValValidator::bool(deactivate_style))
                .with_deactivate_style(deactivate_style))
        })
    }

    pub fn pos() -> Self {
        let type_name = astr("p");

        Self::new(type_name.clone(), false, move |mut config: OptConfig| {
            let deactivate_style = config.deactivate().unwrap_or(false);
            let optional = config.take_optional().unwrap_or(true);
            let assoc = config.take_assoc().unwrap_or(Assoc::Noa);
            let action = config.take_action().unwrap_or(Action::App);
            let initiator = config
                .take_initiator()
                .unwrap_or_else(ValInitiator::empty::<bool>);
            let validator = config.take_validator().unwrap_or_else(ValValidator::some);

            if let Some(v) = config.alias() {
                debug_assert!(v.is_empty(), "Pos option not support alias configruation")
            }
            debug_assert!(
                config.prefix().is_none(),
                "Pos option not support prefix configruation"
            );
            debug_assert!(
                !deactivate_style,
                "Pos option not support deactivate style configuration"
            );
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
                .with_opt_help(config.gen_opt_help(deactivate_style)?)
                .with_optional(optional)
                .with_initiator(initiator)
                .with_validator(validator)
                .with_ignore_name())
        })
    }

    pub fn cmd() -> Self {
        let type_name = astr("c");

        Self::new(type_name.clone(), false, move |mut config: OptConfig| {
            let deactivate_style = config.deactivate().unwrap_or(false);
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
                !config.optional().unwrap_or(false),
                "Cmd option only have default optional configuration"
            );

            debug_assert!(
                config.idx().is_none(),
                "Cmd option only have default index configuration"
            );
            debug_assert!(
                config.prefix().is_none(),
                "Cmd option not support prefix configruation"
            );
            debug_assert!(
                !deactivate_style,
                "Cmd option not support deactivate style configuration"
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
                .with_opt_help(config.gen_opt_help(deactivate_style)?)
                .with_optional(false)
                .with_initiator(initiator)
                .with_validator(validator))
        })
    }

    pub fn main() -> Self {
        let type_name = astr("m");

        Self::new(type_name.clone(), false, move |mut config: OptConfig| {
            let deactivate_style = config.deactivate().unwrap_or(false);
            let assoc = config.take_assoc().unwrap_or(Assoc::Null);
            let action = config.take_action().unwrap_or(Action::Set);
            let initiator = config.take_initiator().unwrap_or_default();
            let validator = config.take_validator().unwrap_or_else(ValValidator::null);

            if let Some(v) = config.alias() {
                debug_assert!(v.is_empty(), "Main option not support alias configruation")
            }
            debug_assert!(
                !config.optional().unwrap_or(false),
                "Main option only have default optional configuration"
            );
            debug_assert!(
                config.idx().is_none(),
                "Main option only have default index configuration"
            );
            debug_assert!(
                config.prefix().is_none(),
                "Main option not support prefix configruation: {:?}",
                config.prefix()
            );
            debug_assert!(
                !deactivate_style,
                "Main option not support deactivate style configuration"
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
                .with_opt_help(config.gen_opt_help(deactivate_style)?)
                .with_optional(true)
                .with_initiator(initiator)
                .with_validator(validator)
                .with_ignore_name())
        })
    }
}
