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
use crate::simple_impl_noa;
use crate::Str;
use crate::Uid;

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct PosOpt {
    uid: Uid,

    name: Str,

    help: OptHelp,

    #[serde(skip)]
    setted: bool,

    index: Option<OptIndex>,

    optional: bool,

    #[serde(skip)]
    callback: Option<OptCallback<Self>>,
}

impl PosOpt {
    pub fn type_name() -> Str {
        astr("p")
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

    pub fn with_optional(mut self, optional: bool) -> Self {
        self.optional = optional;
        self
    }

    pub fn with_callback(mut self, callback: Option<OptCallback<Self>>) -> Self {
        self.callback = callback;
        self
    }

    fn pri_name_mat(&self, _name: &Str) -> bool {
        true
    }

    fn pri_optional_set(&mut self, optional: bool) {
        self.optional = optional;
    }

    fn pri_optional_get(&self) -> bool {
        self.optional
    }

    fn pri_index_set(&self, _index: Option<OptIndex>) {}

    fn pri_index_mat(&self, index: Option<(usize, usize)>) -> bool {
        if let Some((index, total)) = index {
            if let Some(realindex) = self._get_index() {
                if let Some(realindex) = realindex.calc_index(index, total) {
                    return realindex == index;
                }
            }
        }
        false
    }
}

simple_impl_noa!(
    PosOpt,
    Self::type_name(),
    [OptStyle::Pos],
    &Self::pri_name_mat,
    &Self::pri_optional_get,
    &Self::pri_optional_set,
    &Self::pri_index_set,
    &Self::pri_index_mat
);

#[derive(Debug, Default, Clone)]
pub struct PosCreator;

impl PosCreator {
    pub fn boxed() -> Box<PosCreator> {
        Box::new(Self {})
    }
}

impl ACreator for PosCreator {
    type Opt = Box<dyn AOpt>;

    type Config = OptConfig;

    fn _get_type_name(&self) -> Str {
        PosOpt::type_name()
    }

    fn _support_deactivate_style(&self) -> bool {
        false
    }

    fn _create_with(&mut self, config: Self::Config) -> Result<Self::Opt, Error> {
        let deactivate_style = config.deact().unwrap_or(false);

        if deactivate_style && !self._support_deactivate_style() {
            return Err(Error::con_unsupport_deactivate_style(config.gen_name()?));
        }
        if let Some(ty) = config.ty() {
            debug_assert_eq!(ty, &self._get_type_name())
        }

        let opt: PosOpt = config.try_into()?;

        Ok(Box::new(opt))
    }
}

impl TryFrom<OptConfig> for PosOpt {
    type Error = Error;

    fn try_from(mut cfg: OptConfig) -> Result<Self, Self::Error> {
        if let Some(v) = cfg.alias() {
            debug_assert!(v.is_empty(), "Pos option not support alias configruation")
        }
        debug_assert!(
            cfg.pre().is_none(),
            "Pos option not support prefix configruation"
        );
        debug_assert!(
            !cfg.deact().unwrap_or(false),
            "Pos option not support deactivate style configuration"
        );
        Ok(Self::default()
            .with_uid(cfg.uid())
            .with_name(cfg.gen_name()?)
            .with_help(cfg.gen_opt_help(false)?)
            .with_index(Some(cfg.gen_idx()?))
            .with_callback(cfg.take_callback())
            .with_optional(cfg.opt().unwrap_or(true)))
    }
}
