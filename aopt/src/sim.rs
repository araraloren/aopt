use crate::aopt;
use crate::aopt::ACreator;
use crate::aopt::AOpt;
use crate::ext::ASetConfigExt;
use crate::ext::ASetExt;
use crate::opt::OptConfig;
use crate::opt::OptStringParser;
use crate::ser::Services;
use crate::set::OptSet;

pub type SimOpt = Box<dyn AOpt>;

pub type SimCtor = Box<dyn ACreator<Opt = SimOpt, Config = OptConfig>>;

pub type SimSer = Services;

/// Simple option set type.
///
/// Default prefixs are `-` and `--`.
///
/// Default creators:
/// - BoolCreator
/// - FltCreator
/// - IntCreator
/// - StrCreator
/// - UintCreator
/// - CmdCreator
/// - PosCreator
/// - MainCreator
///
/// # Examples
/// ```rust
/// # use aopt::prelude::*;
/// #
/// # fn main() {
///     let set = SimSet::new_set();
///
///     assert_eq!(set.pre(), &[astr("--"), astr("-")]);
///     assert!(set.has_ctor("b"));
///     assert!(set.has_ctor("f"));
///     assert!(set.has_ctor("i"));
///     assert!(set.has_ctor("u"));
///     assert!(set.has_ctor("s"));
///     assert!(set.has_ctor("c"));
///     assert!(set.has_ctor("p"));
///     assert!(set.has_ctor("m"));
/// # }
/// ```
pub type SimSet = OptSet<SimOpt, OptStringParser, SimCtor>;

impl ASetConfigExt for SimSet {
    fn with_default_prefix(mut self) -> Self {
        self = self.with_pre("--").with_pre("-");
        self
    }

    fn with_default_creator(mut self) -> Self {
        self = self
            .with_ctor(aopt::BoolCreator::boxed())
            .with_ctor(aopt::FltCreator::boxed())
            .with_ctor(aopt::IntCreator::boxed())
            .with_ctor(aopt::StrCreator::boxed())
            .with_ctor(aopt::UintCreator::boxed())
            .with_ctor(aopt::CmdCreator::boxed())
            .with_ctor(aopt::PosCreator::boxed())
            .with_ctor(aopt::MainCreator::boxed());
        self
    }
}

impl ASetExt for SimSet {
    fn new_set() -> Self {
        Self::default().with_default_prefix().with_default_creator()
    }
}
