use super::ACreator;
use super::AOpt;
use crate::ctx::Ctx;
use crate::err::Error;
use crate::opt::ConfigValue;
use crate::opt::OptCallback;
use crate::opt::OptConfig;
use crate::opt::OptHelp;
use crate::opt::OptIndex;
use crate::opt::OptStyle;
use crate::ser::Services;
use crate::simple_impl_opt;
use crate::Str;
use crate::Uid;

#[derive(Debug, Default)]
pub struct IntOpt {
    uid: Uid,

    name: Str,

    help: OptHelp,

    prefix: Option<Str>,

    setted: bool,

    optional: bool,

    alias: Vec<(Str, Str)>,

    callback: Option<OptCallback<Self>>,
}

impl IntOpt {
    pub fn type_name() -> Str {
        crate::astr("i")
    }

    pub fn with_uid(mut self, uid: Uid) -> Self {
        self.uid = uid;
        self
    }

    pub fn with_name(mut self, name: Str) -> Self {
        self.name = name;
        self
    }

    pub fn with_help(mut self, help: OptHelp) -> Self {
        self.help = help;
        self
    }

    pub fn with_prefix(mut self, prefix: Option<Str>) -> Self {
        self.prefix = prefix;
        self
    }

    pub fn with_setted(mut self, setted: bool) -> Self {
        self.setted = setted;
        self
    }

    pub fn with_alias(mut self, alias: Vec<(Str, Str)>) -> Self {
        self.alias = alias;
        self
    }

    pub fn with_optional(mut self, optional: bool) -> Self {
        self.optional = optional;
        self
    }

    pub fn with_callback(mut self, callback: Option<OptCallback<Self>>) -> Self {
        self.callback = callback;
        self
    }

    fn pri_check(
        &mut self,
        arg: Option<Str>,
        _disable: bool,
        _index: (usize, usize),
    ) -> Result<bool, Error> {
        arg.ok_or_else(|| Error::sp_missing_argument(self._get_name()))?
            .parse::<i64>()
            .map_err(|e| {
                Error::sp_invalid_option_value(self._get_name().to_string(), e.to_string())
            })?;
        Ok(true)
    }

    fn pri_is_deactivate_style(&self) -> bool {
        false
    }
}

simple_impl_opt!(
    IntOpt,
    Self::type_name(),
    [OptStyle::Argument],
    &Self::pri_check,
    &Self::pri_is_deactivate_style
);

#[derive(Debug, Default, Clone)]
pub struct IntCreator;

impl IntCreator {
    pub fn boxed() -> Box<IntCreator> {
        Box::new(Self {})
    }
}

impl ACreator for IntCreator {
    type Opt = Box<dyn AOpt>;

    type Config = OptConfig;

    fn _get_type_name(&self) -> Str {
        IntOpt::type_name()
    }

    fn _support_deactivate_style(&self) -> bool {
        false
    }

    fn _create_with(&mut self, config: Self::Config) -> Result<Self::Opt, Error> {
        let deactivate_style = config.deact().unwrap_or(false);

        if deactivate_style && !self._support_deactivate_style() {
            return Err(Error::con_unsupport_deactivate_style(config.gen_name()?));
        }

        debug_assert_eq!(config.ty().unwrap(), self._get_type_name());

        let opt: IntOpt = config.try_into()?;

        Ok(Box::new(opt))
    }
}

impl TryFrom<OptConfig> for IntOpt {
    type Error = Error;

    fn try_from(mut cfg: OptConfig) -> Result<Self, Self::Error> {
        let prefix = Some(cfg.gen_pre()?);
        let optional = cfg.take_opt().unwrap_or(true);

        debug_assert!(
            cfg.idx().is_none(),
            "Flt option not support index configruation"
        );
        debug_assert!(
            !cfg.deact().unwrap_or(false),
            "Flt option not support deactivate style configuration"
        );
        Ok(Self::default()
            .with_uid(cfg.gen_uid())
            .with_name(cfg.gen_name()?)
            .with_prefix(prefix)
            .with_help(cfg.gen_opt_help(false)?)
            .with_alias(cfg.gen_alias()?)
            .with_optional(optional)
            .with_callback(cfg.take_callback()))
    }
}
