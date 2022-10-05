use super::ACreator;
use super::AOpt;
use crate::ctx::Context;
use crate::err::Error;
use crate::opt::ConfigValue;
use crate::opt::OptCallback;
use crate::opt::OptConfig;
use crate::opt::OptHelp;
use crate::opt::OptIndex;
use crate::opt::OptStyle;
use crate::ser::Services;
use crate::simple_impl_noa;
use crate::Str;
use crate::Uid;

#[derive(Debug, Default)]
pub struct MainOpt {
    uid: Uid,

    name: Str,

    help: OptHelp,

    setted: bool,

    index: Option<OptIndex>,

    callback: Option<OptCallback<Self>>,
}

impl MainOpt {
    pub fn type_name() -> Str {
        crate::astr("m")
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

    pub fn with_setted(mut self, setted: bool) -> Self {
        self.setted = setted;
        self
    }

    pub fn with_index(mut self, index: Option<OptIndex>) -> Self {
        self.index = index;
        self
    }

    pub fn with_callback(mut self, callback: Option<OptCallback<Self>>) -> Self {
        self.callback = callback;
        self
    }

    fn pri_name_mat(&self, _name: Str) -> bool {
        true
    }

    fn pri_optional_set(&self, _: bool) {}

    fn pri_optional_get(&self) -> bool {
        true
    }

    fn pri_index_set(&self, _index: Option<OptIndex>) {}

    fn pri_index_mat(&self, index: Option<(usize, usize)>) -> bool {
        index.is_some()
    }
}

simple_impl_noa!(
    MainOpt,
    Self::type_name(),
    [OptStyle::Main],
    &Self::pri_name_mat,
    &Self::pri_optional_get,
    &Self::pri_optional_set,
    &Self::pri_index_set,
    &Self::pri_index_mat
);

#[derive(Debug, Default, Clone)]
pub struct MainCreator;

impl MainCreator {
    pub fn boxed() -> Box<MainCreator> {
        Box::new(Self {})
    }
}

impl ACreator for MainCreator {
    type Opt = Box<dyn AOpt>;

    type Config = OptConfig;

    fn _get_type_name(&self) -> Str {
        MainOpt::type_name()
    }

    fn _support_deactivate_style(&self) -> bool {
        false
    }

    fn _create_with(&mut self, config: Self::Config) -> Result<Self::Opt, Error> {
        let deactivate_style = config.get_deactivate_style().unwrap_or(false);

        if deactivate_style && !self._support_deactivate_style() {
            return Err(Error::con_unsupport_deactivate_style(config.gen_name()?));
        }

        debug_assert_eq!(config.get_type_name().unwrap(), self._get_type_name());

        let opt: MainOpt = config.try_into()?;

        Ok(Box::new(opt))
    }
}

impl TryFrom<OptConfig> for MainOpt {
    type Error = Error;

    fn try_from(mut cfg: OptConfig) -> Result<Self, Self::Error> {
        if let Some(v) = cfg.get_alias() {
            debug_assert!(v.is_empty(), "Main option not support alias configruation")
        }
        debug_assert!(
            cfg.get_optional().unwrap_or(true),
            "Main option only have default optional configuration"
        );
        debug_assert!(
            cfg.get_index().is_none(),
            "Main option only have default index configuration"
        );
        debug_assert!(
            cfg.get_prefix().is_none(),
            "Main option not support prefix configruation"
        );
        debug_assert!(
            !cfg.get_deactivate_style().unwrap_or(false),
            "Main option not support deactivate style configuration"
        );
        Ok(Self::default()
            .with_uid(cfg.get_uid())
            .with_name(cfg.gen_name()?)
            .with_help(cfg.gen_opt_help(false)?)
            .with_index(None)
            .with_callback(cfg.take_callback()))
    }
}