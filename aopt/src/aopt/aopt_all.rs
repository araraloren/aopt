use super::ACreator;
use super::AOpt;
use crate::astr;
use crate::ctx::Ctx;
use crate::err::Error;
use crate::opt::ConfigValue;
use crate::opt::OptConfig;
use crate::opt::OptHelp;
use crate::opt::OptIndex;
use crate::opt::OptStyle;
use crate::opt::ValParser;
use crate::opt::ValPolicy;
use crate::opt::ValType;
use crate::opt::ValValidator;
use crate::ser::Services;
use crate::simple_impl_opt;
use crate::Arc;
use crate::RawVal;
use crate::Str;
use crate::Uid;

pub struct UOpt {
    uid: Uid,

    name: Str,

    r#type: Str,

    help: OptHelp,

    prefix: Option<Str>,

    setted: bool,

    optional: bool,

    valtype: ValType,

    policy: ValPolicy,

    styles: Vec<OptStyle>,

    deactivate_style: bool,

    alias: Vec<(Str, Str)>,

    index: Option<OptIndex>,

    validator: ValValidator,
}
