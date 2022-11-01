use crate::opt::AOpt;
use crate::opt::ConfigValue;
use crate::opt::Creator;
use crate::opt::OptConfig;
use crate::opt::OptStyle;
use crate::opt::ValAction;
use crate::opt::ValAssoc;
use crate::opt::ValValidator;
use crate::Error;
use crate::Str;

#[derive(Debug, Default, Clone)]
pub struct IntCreator(Str);

impl IntCreator {
    pub fn boxed() -> Box<IntCreator> {
        Box::new(Self(Str::from("i")))
    }
}

impl Creator for IntCreator {
    type Opt = AOpt;

    type Config = OptConfig;

    type Error = Error;

    fn r#type(&self) -> Str {
        self.0.clone()
    }

    fn sp_deactivate(&self) -> bool {
        false
    }

    fn new_with(&mut self, mut config: Self::Config) -> Result<Self::Opt, Self::Error> {
        let deactivate_style = config.deactivate().unwrap_or(false);
        let prefix = Some(config.gen_prefix()?);
        let optional = config.take_optional().unwrap_or(true);
        let assoc = config.take_assoc().unwrap_or(ValAssoc::Int);
        let action = config.take_action().unwrap_or(ValAction::App);

        debug_assert_eq!(
            assoc,
            ValAssoc::Int,
            "The type must be ValType::Int for IntCreator"
        );
        debug_assert!(
            config.idx().is_none(),
            "Flt option not support index configruation"
        );
        debug_assert!(
            !config.deactivate().unwrap_or(false),
            "Flt option not support deactivate style configuration"
        );
        if deactivate_style && !self.sp_deactivate() {
            return Err(Error::con_unsupport_deactivate_style(config.gen_name()?));
        }
        if let Some(r#type) = config.r#type() {
            debug_assert_eq!(r#type, &self.r#type())
        }
        Ok(Self::Opt::default()
            .with_type(self.r#type())
            .with_name(config.gen_name()?)
            .with_prefix(prefix)
            .with_assoc(assoc)
            .with_action(action)
            .with_style(vec![OptStyle::Argument])
            .with_opt_help(config.gen_opt_help(false)?)
            .with_alias(Some(config.gen_alias()?))
            .with_optional(optional)
            .with_validator(ValValidator::i64_validator()))
    }
}
