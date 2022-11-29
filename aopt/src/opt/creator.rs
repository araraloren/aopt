use crate::opt::AOpt;
use crate::opt::Action;
use crate::opt::Assoc;
use crate::opt::ConfigValue;
use crate::opt::OptConfig;
use crate::opt::Style;
use crate::opt::ValInitiator;
use crate::opt::ValValidator;
use crate::set::Ctor;
use crate::Error;
use crate::Str;

#[derive(Debug, Default, Clone)]
pub struct IntCreator(Str);

impl IntCreator {
    pub fn boxed() -> Box<IntCreator> {
        Box::new(Self(Str::from("i")))
    }
}

impl Ctor for IntCreator {
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
        let assoc = config.take_assoc().unwrap_or(Assoc::Int);
        let action = config.take_action().unwrap_or(Action::App);
        let initiator = config.take_initiator().unwrap_or_default();

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
            debug_assert_eq!(r#type, &self.r#type())
        }
        Ok(Self::Opt::default()
            .with_type(self.r#type())
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
    }
}

#[derive(Debug, Default, Clone)]
pub struct UintCreator(Str);

impl UintCreator {
    pub fn boxed() -> Box<UintCreator> {
        Box::new(Self(Str::from("u")))
    }
}

impl Ctor for UintCreator {
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
        let assoc = config.take_assoc().unwrap_or(Assoc::Uint);
        let action = config.take_action().unwrap_or(Action::App);
        let initiator = config.take_initiator().unwrap_or_default();

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
            debug_assert_eq!(r#type, &self.r#type())
        }
        Ok(Self::Opt::default()
            .with_type(self.r#type())
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
    }
}

#[derive(Debug, Default, Clone)]
pub struct FltCreator(Str);

impl FltCreator {
    pub fn boxed() -> Box<FltCreator> {
        Box::new(Self(Str::from("f")))
    }
}

impl Ctor for FltCreator {
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
        let assoc = config.take_assoc().unwrap_or(Assoc::Flt);
        let action = config.take_action().unwrap_or(Action::App);
        let initiator = config.take_initiator().unwrap_or_default();

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
            debug_assert_eq!(r#type, &self.r#type())
        }
        Ok(Self::Opt::default()
            .with_type(self.r#type())
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
    }
}

#[derive(Debug, Default, Clone)]
pub struct StrCreator(Str);

impl StrCreator {
    pub fn boxed() -> Box<StrCreator> {
        Box::new(Self(Str::from("s")))
    }
}

impl Ctor for StrCreator {
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
        let assoc = config.take_assoc().unwrap_or(Assoc::Str);
        let action = config.take_action().unwrap_or(Action::App);
        let initiator = config.take_initiator().unwrap_or_default();

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
            debug_assert_eq!(r#type, &self.r#type())
        }
        Ok(Self::Opt::default()
            .with_type(self.r#type())
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
    }
}

#[derive(Debug, Default, Clone)]
pub struct BoolCreator(Str);

impl BoolCreator {
    pub fn boxed() -> Box<BoolCreator> {
        Box::new(Self(Str::from("b")))
    }
}

impl Ctor for BoolCreator {
    type Opt = AOpt;

    type Config = OptConfig;

    type Error = Error;

    fn r#type(&self) -> Str {
        self.0.clone()
    }

    fn sp_deactivate(&self) -> bool {
        true
    }

    fn new_with(&mut self, mut config: Self::Config) -> Result<Self::Opt, Self::Error> {
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
            .with_style(vec![Style::Boolean, Style::Combined])
            .with_opt_help(config.gen_opt_help(deactivate_style)?)
            .with_alias(Some(config.gen_alias()?))
            .with_optional(optional)
            .with_initiator(ValInitiator::bool(value))
            .with_validator(ValValidator::bool(deactivate_style))
            .with_deactivate_style(deactivate_style))
    }
}

#[derive(Debug, Default, Clone)]
pub struct PosCreator(Str);

impl PosCreator {
    pub fn boxed() -> Box<PosCreator> {
        Box::new(Self(Str::from("p")))
    }
}

impl Ctor for PosCreator {
    type Opt = AOpt;

    type Config = OptConfig;

    type Error = Error;

    fn r#type(&self) -> Str {
        self.0.clone()
    }

    fn sp_deactivate(&self) -> bool {
        true
    }

    fn new_with(&mut self, mut config: Self::Config) -> Result<Self::Opt, Self::Error> {
        let deactivate_style = config.deactivate().unwrap_or(false);
        let optional = config.take_optional().unwrap_or(true);
        let assoc = config.take_assoc().unwrap_or(Assoc::Noa);
        let action = config.take_action().unwrap_or(Action::App);
        let initiator = config.take_initiator().unwrap_or_default();
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
            debug_assert_eq!(r#type, &self.r#type())
        }
        Ok(Self::Opt::default()
            .with_type(self.r#type())
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
    }
}

#[derive(Debug, Default, Clone)]
pub struct CmdCreator(Str);

impl CmdCreator {
    pub fn boxed() -> Box<CmdCreator> {
        Box::new(Self(Str::from("c")))
    }
}

impl Ctor for CmdCreator {
    type Opt = AOpt;

    type Config = OptConfig;

    type Error = Error;

    fn r#type(&self) -> Str {
        self.0.clone()
    }

    fn sp_deactivate(&self) -> bool {
        true
    }

    fn new_with(&mut self, mut config: Self::Config) -> Result<Self::Opt, Self::Error> {
        let deactivate_style = config.deactivate().unwrap_or(false);
        let assoc = config.take_assoc().unwrap_or(Assoc::Noa);
        let action = config.take_action().unwrap_or(Action::Set);
        let initiator = config.take_initiator().unwrap_or_default();
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
            debug_assert_eq!(r#type, &self.r#type())
        }
        Ok(Self::Opt::default()
            .with_type(self.r#type())
            .with_name(config.gen_name()?)
            .with_assoc(assoc)
            .with_action(action)
            .with_idx(Some(crate::opt::Index::forward(1)))
            .with_style(vec![Style::Cmd])
            .with_opt_help(config.gen_opt_help(deactivate_style)?)
            .with_optional(false)
            .with_initiator(initiator)
            .with_validator(validator))
    }
}

#[derive(Debug, Default, Clone)]
pub struct MainCreator(Str);

impl MainCreator {
    pub fn boxed() -> Box<MainCreator> {
        Box::new(Self(Str::from("m")))
    }
}

impl Ctor for MainCreator {
    type Opt = AOpt;

    type Config = OptConfig;

    type Error = Error;

    fn r#type(&self) -> Str {
        self.0.clone()
    }

    fn sp_deactivate(&self) -> bool {
        true
    }

    fn new_with(&mut self, mut config: Self::Config) -> Result<Self::Opt, Self::Error> {
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
            debug_assert_eq!(r#type, &self.r#type())
        }
        Ok(Self::Opt::default()
            .with_type(self.r#type())
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
    }
}
