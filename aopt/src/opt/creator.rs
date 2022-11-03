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
            .with_style(vec![OptStyle::Argument])
            .with_opt_help(config.gen_opt_help(false)?)
            .with_alias(Some(config.gen_alias()?))
            .with_optional(optional)
            .with_validator(ValValidator::i64_validator()))
    }
}

#[derive(Debug, Default, Clone)]
pub struct UintCreator(Str);

impl UintCreator {
    pub fn boxed() -> Box<UintCreator> {
        Box::new(Self(Str::from("u")))
    }
}

impl Creator for UintCreator {
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
        let assoc = config.take_assoc().unwrap_or(ValAssoc::Uint);
        let action = config.take_action().unwrap_or(ValAction::App);

        debug_assert_eq!(
            assoc,
            ValAssoc::Uint,
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
            .with_style(vec![OptStyle::Argument])
            .with_opt_help(config.gen_opt_help(false)?)
            .with_alias(Some(config.gen_alias()?))
            .with_optional(optional)
            .with_validator(ValValidator::u64_validator()))
    }
}

#[derive(Debug, Default, Clone)]
pub struct FltCreator(Str);

impl FltCreator {
    pub fn boxed() -> Box<FltCreator> {
        Box::new(Self(Str::from("f")))
    }
}

impl Creator for FltCreator {
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
        let assoc = config.take_assoc().unwrap_or(ValAssoc::Flt);
        let action = config.take_action().unwrap_or(ValAction::App);

        debug_assert_eq!(
            assoc,
            ValAssoc::Flt,
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
            .with_style(vec![OptStyle::Argument])
            .with_opt_help(config.gen_opt_help(false)?)
            .with_alias(Some(config.gen_alias()?))
            .with_optional(optional)
            .with_validator(ValValidator::f64_validator()))
    }
}

#[derive(Debug, Default, Clone)]
pub struct StrCreator(Str);

impl StrCreator {
    pub fn boxed() -> Box<StrCreator> {
        Box::new(Self(Str::from("s")))
    }
}

impl Creator for StrCreator {
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
        let assoc = config.take_assoc().unwrap_or(ValAssoc::Str);
        let action = config.take_action().unwrap_or(ValAction::App);

        debug_assert_eq!(
            assoc,
            ValAssoc::Str,
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
            .with_style(vec![OptStyle::Argument])
            .with_opt_help(config.gen_opt_help(false)?)
            .with_alias(Some(config.gen_alias()?))
            .with_optional(optional)
            .with_validator(ValValidator::str_validator()))
    }
}

#[derive(Debug, Default, Clone)]
pub struct BoolCreator(Str);

impl BoolCreator {
    pub fn boxed() -> Box<BoolCreator> {
        Box::new(Self(Str::from("b")))
    }
}

impl Creator for BoolCreator {
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
        let assoc = config.take_assoc().unwrap_or(ValAssoc::Bool);
        let action = config.take_action().unwrap_or(ValAction::Set);

        debug_assert_eq!(
            assoc,
            ValAssoc::Bool,
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
            .with_style(vec![OptStyle::Boolean, OptStyle::Combined])
            .with_opt_help(config.gen_opt_help(deactivate_style)?)
            .with_alias(Some(config.gen_alias()?))
            .with_optional(optional)
            .with_validator(ValValidator::bool_validator(deactivate_style)))
    }
}

#[derive(Debug, Default, Clone)]
pub struct PosCreator(Str);

impl PosCreator {
    pub fn boxed() -> Box<PosCreator> {
        Box::new(Self(Str::from("p")))
    }
}

impl Creator for PosCreator {
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
        let assoc = config.take_assoc().unwrap_or(ValAssoc::Bool);
        let action = config.take_action().unwrap_or(ValAction::App);
        let validator = config
            .take_validator()
            .unwrap_or(ValValidator::bool_validator(false));

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
            .with_style(vec![OptStyle::Pos])
            .with_opt_help(config.gen_opt_help(deactivate_style)?)
            .with_optional(optional)
            .with_validator(validator))
    }
}

#[derive(Debug, Default, Clone)]
pub struct CmdCreator(Str);

impl CmdCreator {
    pub fn boxed() -> Box<CmdCreator> {
        Box::new(Self(Str::from("c")))
    }
}

impl Creator for CmdCreator {
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
        let assoc = config.take_assoc().unwrap_or(ValAssoc::Bool);
        let action = config.take_action().unwrap_or(ValAction::App);
        let validator = config
            .take_validator()
            .unwrap_or(ValValidator::bool_validator(false));

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
            .with_idx(Some(crate::opt::OptIndex::forward(1)))
            .with_style(vec![OptStyle::Cmd])
            .with_opt_help(config.gen_opt_help(deactivate_style)?)
            .with_optional(false)
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

impl Creator for MainCreator {
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
        let assoc = config.take_assoc().unwrap_or(ValAssoc::Null);
        let action = config.take_action().unwrap_or(ValAction::Null);
        let validator = config
            .take_validator()
            .unwrap_or(ValValidator::null_validator());

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
            "Main option not support prefix configruation"
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
            .with_idx(Some(crate::opt::OptIndex::anywhere()))
            .with_style(vec![OptStyle::Main])
            .with_opt_help(config.gen_opt_help(deactivate_style)?)
            .with_optional(true)
            .with_validator(validator))
    }
}
