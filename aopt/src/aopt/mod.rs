pub(crate) mod aopt_bool;
pub(crate) mod aopt_cmd;
pub(crate) mod aopt_flt;
pub(crate) mod aopt_int;
pub(crate) mod aopt_main;
pub(crate) mod aopt_pos;
pub(crate) mod aopt_str;
pub(crate) mod aopt_uint;
pub(crate) mod simple_macro;

pub use self::aopt_bool::BoolCreator;
pub use self::aopt_bool::BoolOpt;
pub use self::aopt_cmd::CmdCreator;
pub use self::aopt_cmd::CmdOpt;
pub use self::aopt_flt::FltCreator;
pub use self::aopt_flt::FltOpt;
pub use self::aopt_int::IntCreator;
pub use self::aopt_int::IntOpt;
pub use self::aopt_main::MainCreator;
pub use self::aopt_main::MainOpt;
pub use self::aopt_pos::PosCreator;
pub use self::aopt_pos::PosOpt;
pub use self::aopt_str::StrCreator;
pub use self::aopt_str::StrOpt;
pub use self::aopt_uint::UintCreator;
pub use self::aopt_uint::UintOpt;

use std::fmt::Debug;

use crate::ctx::Ctx;
use crate::err::Error;
use crate::opt::Alias;
use crate::opt::Creator;
use crate::opt::Help;
use crate::opt::Index;
use crate::opt::Name;
use crate::opt::Opt;
use crate::opt::OptConfig;
use crate::opt::OptIndex;
use crate::opt::OptStyle;
use crate::opt::Optional;
use crate::opt::Prefix;
use crate::ser::RawValService;
use crate::ser::Services;
use crate::simple_impl_creator_for;
use crate::simple_impl_opt_for;
use crate::Arc;
use crate::RawVal;
use crate::Str;
use crate::Uid;

pub trait AOpt: Debug {
    fn _reset(&mut self) {
        self._set_setted(false);
    }

    fn _valid(&self) -> bool {
        self._get_optional() || self._get_setted()
    }

    fn _get_uid(&self) -> Uid;

    fn _set_uid(&mut self, uid: Uid);

    fn _get_setted(&self) -> bool;

    fn _set_setted(&mut self, setted: bool);

    fn _get_type_name(&self) -> Str;

    fn _is_deactivate_style(&self) -> bool;

    fn _match_style(&self, style: OptStyle) -> bool;

    fn _get_name(&self) -> &Str;

    fn _set_name(&mut self, name: Str);

    fn _match_name(&self, name: &Str) -> bool {
        self._get_name() == name
    }

    fn _get_prefix(&self) -> Option<&Str>;

    fn _set_prefix(&mut self, prefix: Option<Str>);

    fn _match_prefix(&self, prefix: Option<&Str>) -> bool {
        self._get_prefix() == prefix
    }

    fn _get_optional(&self) -> bool;

    fn _set_optional(&mut self, optional: bool);

    fn _match_optional(&self, optional: bool) -> bool {
        self._get_optional() == optional
    }

    fn _get_alias(&self) -> Option<&Vec<(Str, Str)>>;

    fn _add_alias(&mut self, prefix: Str, name: Str);

    fn _rem_alias(&mut self, prefix: &Str, name: &Str);

    fn _match_alias(&self, prefix: &Str, name: &Str) -> bool;

    fn _get_hint(&self) -> &Str;

    fn _get_help(&self) -> &Str;

    fn _set_hint(&mut self, hint: Str);

    fn _set_help(&mut self, help: Str);

    fn _get_index(&self) -> Option<&OptIndex>;

    fn _set_index(&mut self, index: Option<OptIndex>);

    fn _match_index(&self, index: Option<(usize, usize)>) -> bool;

    fn _check(
        &mut self,
        val: Option<Arc<RawVal>>,
        disable: bool,
        index: (usize, usize),
    ) -> Result<bool, Error>;

    fn _val_act(&mut self, val: Option<RawVal>, ser: &mut Services, ctx: &Ctx)
        -> Result<(), Error>;
}

simple_impl_opt_for!(BoolOpt);
simple_impl_opt_for!(FltOpt);
simple_impl_opt_for!(IntOpt);
simple_impl_opt_for!(StrOpt);
simple_impl_opt_for!(UintOpt);
simple_impl_opt_for!(CmdOpt);
simple_impl_opt_for!(MainOpt);
simple_impl_opt_for!(PosOpt);
simple_impl_opt_for!(Box<dyn AOpt>);

pub trait ACreator {
    type Opt;
    type Config;

    fn _get_type_name(&self) -> Str;

    fn _support_deactivate_style(&self) -> bool;

    fn _create_with(&mut self, config: Self::Config) -> Result<Self::Opt, Error>;
}

impl<Opt, Config> std::fmt::Debug for Box<dyn ACreator<Opt = Opt, Config = Config>> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Box")
            .field(&format!("ACreator({})", self.ty()))
            .finish()
    }
}

impl<Opt, Config> Creator for Box<dyn ACreator<Opt = Opt, Config = Config>> {
    type Opt = Opt;

    type Config = Config;

    type Error = Error;

    fn ty(&self) -> Str {
        self._get_type_name()
    }

    fn sp_deact(&self) -> bool {
        self._support_deactivate_style()
    }

    fn new_with(&mut self, config: Self::Config) -> Result<Self::Opt, Self::Error> {
        self._create_with(config)
    }
}

simple_impl_creator_for!(BoolCreator);
simple_impl_creator_for!(FltCreator);
simple_impl_creator_for!(IntCreator);
simple_impl_creator_for!(StrCreator);
simple_impl_creator_for!(UintCreator);
simple_impl_creator_for!(CmdCreator);
simple_impl_creator_for!(MainCreator);
simple_impl_creator_for!(PosCreator);

/// Adding convert into OptConfig support serialize and deserialize.
impl From<Box<dyn AOpt>> for OptConfig {
    fn from(v: Box<dyn AOpt>) -> Self {
        Self::from(&v)
    }
}

impl<'a> From<&'a Box<dyn AOpt>> for OptConfig {
    fn from(v: &'a Box<dyn AOpt>) -> Self {
        let mut cfg = OptConfig::default()
            .with_uid(v._get_uid())
            .with_name(v._get_name())
            .with_pre(v._get_prefix())
            .with_opt(v._get_optional())
            .with_help(v._get_help())
            .with_hint(v._get_hint())
            .with_ty(v._get_type_name())
            .with_deact(v._is_deactivate_style());
        if let Some(alias) = v._get_alias() {
            cfg = cfg.with_alias(
                alias
                    .iter()
                    .map(|v| Str::from(format!("{}{}", v.0, v.1)))
                    .collect(),
            );
        }
        if let Some(idx) = v._get_index() {
            cfg = cfg.with_idx(idx.clone());
        }
        cfg
    }
}
