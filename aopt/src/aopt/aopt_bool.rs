use super::ACreator;
use super::AOpt;
use crate::astr;
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
use crate::Arc;
use crate::RawString;
use crate::Str;
use crate::Uid;

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct BoolOpt {
    uid: Uid,

    name: Str,

    help: OptHelp,

    prefix: Option<Str>,

    #[serde(skip)]
    setted: bool,

    optional: bool,

    deactivate_style: bool,

    alias: Vec<(Str, Str)>,

    #[serde(skip)]
    callback: Option<OptCallback<Self>>,
}

impl BoolOpt {
    pub fn type_name() -> Str {
        astr("b")
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

    pub fn with_deactivate_style(mut self, deactivate_style: bool) -> Self {
        self.deactivate_style = deactivate_style;
        self
    }

    pub fn with_callback(mut self, callback: Option<OptCallback<Self>>) -> Self {
        self.callback = callback;
        self
    }

    fn pri_check(
        &mut self,
        _arg: Option<Arc<RawString>>,
        disable: bool,
        _index: (usize, usize),
    ) -> Result<bool, Error> {
        if !self._is_deactivate_style() && disable {
            Err(Error::sp_not_support_deactivate(self._get_name()))
        } else {
            Ok(true)
        }
    }

    fn pri_is_deactivate_style(&self) -> bool {
        self.deactivate_style
    }
}

simple_impl_opt!(
    BoolOpt,
    Self::type_name(),
    [OptStyle::Boolean, OptStyle::Combined],
    &Self::pri_check,
    &Self::pri_is_deactivate_style
);

#[derive(Debug, Default, Clone)]
pub struct BoolCreator;

impl BoolCreator {
    pub fn boxed() -> Box<BoolCreator> {
        Box::new(Self::default())
    }
}

impl ACreator for BoolCreator {
    type Opt = Box<dyn AOpt>;

    type Config = OptConfig;

    fn _get_type_name(&self) -> Str {
        BoolOpt::type_name()
    }

    fn _support_deactivate_style(&self) -> bool {
        true
    }

    fn _create_with(&mut self, config: Self::Config) -> Result<Self::Opt, Error> {
        let deactivate_style = config.deact().unwrap_or(false);

        if deactivate_style && !self._support_deactivate_style() {
            return Err(Error::con_unsupport_deactivate_style(config.gen_name()?));
        }
        if let Some(ty) = config.ty() {
            debug_assert_eq!(ty, &self._get_type_name())
        }

        let opt: BoolOpt = config.try_into()?;

        Ok(Box::new(opt))
    }
}

impl TryFrom<OptConfig> for BoolOpt {
    type Error = Error;

    fn try_from(mut cfg: OptConfig) -> Result<Self, Self::Error> {
        let prefix = Some(cfg.gen_pre()?);
        let deactivate_style = cfg.deact().unwrap_or(false);
        let optional = cfg.take_opt().unwrap_or(true);

        debug_assert!(
            cfg.idx().is_none(),
            "Bool option not support index configruation"
        );
        Ok(Self::default()
            .with_uid(cfg.gen_uid())
            .with_name(cfg.gen_name()?)
            .with_prefix(prefix)
            .with_help(cfg.gen_opt_help(deactivate_style)?)
            .with_alias(cfg.gen_alias()?)
            .with_optional(optional)
            .with_callback(cfg.take_callback())
            .with_deactivate_style(deactivate_style))
    }
}
