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
}
