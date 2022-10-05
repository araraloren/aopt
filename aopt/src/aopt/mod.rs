pub(crate) mod aopt_bool;
pub(crate) mod aopt_cmd;
pub(crate) mod aopt_flt;
pub(crate) mod aopt_int;
pub(crate) mod aopt_main;
pub(crate) mod aopt_pos;
pub(crate) mod aopt_str;
pub(crate) mod aopt_uint;
pub(crate) mod creator;
pub(crate) mod data;
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
pub use self::creator::ACreator;
pub use self::data::UserData;

use std::fmt::Debug;

use crate::ctx::Context;
use crate::err::Error;
use crate::opt::Alias;
use crate::opt::Help;
use crate::opt::Index;
use crate::opt::Name;
use crate::opt::Opt;
use crate::opt::OptIndex;
use crate::opt::OptStyle;
use crate::opt::Optional;
use crate::opt::Prefix;
use crate::prelude::Services;
use crate::Str;
use crate::Uid;

pub trait AOpt: Debug {
    fn _reset(&mut self) {
        self._set_setted(false);
    }

    fn _check(&self) -> bool {
        self._get_optional() || self._get_setted()
    }

    fn _get_uid(&self) -> Uid;

    fn _set_uid(&mut self, uid: Uid);

    fn _get_setted(&self) -> bool;

    fn _set_setted(&mut self, setted: bool);

    fn _get_type_name(&self) -> Str;

    fn _is_deactivate_style(&self) -> bool;

    fn _match_style(&self, style: OptStyle) -> bool;

    fn _get_name(&self) -> Str;

    fn _set_name(&mut self, name: Str);

    fn _match_name(&self, name: Str) -> bool {
        self._get_name() == name
    }

    fn _get_prefix(&self) -> Option<Str>;

    fn _set_prefix(&mut self, prefix: Option<Str>);

    fn _match_prefix(&self, prefix: Option<Str>) -> bool {
        self._get_prefix() == prefix
    }

    fn _get_optional(&self) -> bool;

    fn _set_optional(&mut self, optional: bool);

    fn _match_optional(&self, optional: bool) -> bool {
        self._get_optional() == optional
    }

    fn _get_alias(&self) -> Option<&Vec<(Str, Str)>>;

    fn _add_alias(&mut self, prefix: Str, name: Str);

    fn _rem_alias(&mut self, prefix: Str, name: Str);

    fn _match_alias(&self, prefix: Str, name: Str) -> bool;

    fn _get_hint(&self) -> Str;

    fn _get_help(&self) -> Str;

    fn _set_hint(&mut self, hint: Str);

    fn _set_help(&mut self, help: Str);

    fn _get_index(&self) -> Option<&OptIndex>;

    fn _set_index(&mut self, index: Option<OptIndex>);

    fn _match_index(&self, index: Option<(usize, usize)>) -> bool;

    fn _chk_value(
        &mut self,
        arg: Option<Str>,
        disable: bool,
        index: (usize, usize),
    ) -> Result<bool, Error>;

    fn _has_callback(&self) -> bool;

    fn _invoke(&mut self, ctx: &Context, ser: &mut Services) -> Result<Option<Str>, Error>;
}

impl Opt for Box<dyn AOpt> {
    fn reset(&mut self) {
        self._reset()
    }

    fn check(&self) -> bool {
        self._check()
    }

    fn get_uid(&self) -> Uid {
        self._get_uid()
    }

    fn set_uid(&mut self, uid: Uid) {
        self._set_uid(uid)
    }

    fn set_setted(&mut self, setted: bool) {
        self._set_setted(setted)
    }

    fn get_setted(&self) -> bool {
        self._get_setted()
    }

    fn get_type_name(&self) -> Str {
        self._get_type_name()
    }

    fn is_deactivate_style(&self) -> bool {
        self._is_deactivate_style()
    }

    fn match_style(&self, style: OptStyle) -> bool {
        self._match_style(style)
    }

    fn has_callback(&self) -> bool {
        self._has_callback()
    }

    fn invoke_callback(&mut self, ctx: &Context, ser: &mut Services) -> Result<Option<Str>, Error> {
        self._invoke(ctx, ser)
    }

    fn check_value(
        &mut self,
        arg: Option<Str>,
        disable: bool,
        index: (usize, usize),
    ) -> Result<bool, Error> {
        self._chk_value(arg, disable, index)
    }
}

impl Name for Box<dyn AOpt> {
    fn get_name(&self) -> Str {
        self._get_name()
    }

    fn set_name(&mut self, name: Str) {
        self._set_name(name)
    }

    fn match_name(&self, name: Str) -> bool {
        self._match_name(name)
    }
}

impl Prefix for Box<dyn AOpt> {
    fn get_prefix(&self) -> Option<Str> {
        self._get_prefix()
    }

    fn set_prefix(&mut self, prefix: Option<Str>) {
        self._set_prefix(prefix)
    }

    fn match_prefix(&self, prefix: Option<Str>) -> bool {
        self._match_prefix(prefix)
    }
}

impl Optional for Box<dyn AOpt> {
    fn get_optional(&self) -> bool {
        self._get_optional()
    }

    fn set_optional(&mut self, optional: bool) {
        self._set_optional(optional)
    }

    fn match_optional(&self, optional: bool) -> bool {
        self._match_optional(optional)
    }
}

impl Alias for Box<dyn AOpt> {
    fn get_alias(&self) -> Option<&Vec<(Str, Str)>> {
        self._get_alias()
    }

    fn add_alias(&mut self, prefix: Str, name: Str) {
        self._add_alias(prefix, name)
    }

    fn rem_alias(&mut self, prefix: Str, name: Str) {
        self._rem_alias(prefix, name)
    }

    fn match_alias(&self, prefix: Str, name: Str) -> bool {
        self._match_alias(prefix, name)
    }
}

impl Help for Box<dyn AOpt> {
    fn get_hint(&self) -> Str {
        self._get_hint()
    }

    fn get_help(&self) -> Str {
        self._get_help()
    }

    fn set_hint(&mut self, hint: Str) {
        self._set_hint(hint)
    }

    fn set_help(&mut self, help: Str) {
        self._set_help(help)
    }
}

impl Index for Box<dyn AOpt> {
    fn get_index(&self) -> Option<&OptIndex> {
        self._get_index()
    }

    fn set_index(&mut self, index: Option<OptIndex>) {
        self._set_index(index)
    }

    fn match_index(&self, index: Option<(usize, usize)>) -> bool {
        self._match_index(index)
    }
}
