pub(crate) mod infer;

use std::ffi::OsStr;
use std::fmt::Debug;

pub use self::infer::Infer;
pub use self::infer::Placeholder;

pub use crate::acore::value::raw2str;
pub use crate::acore::value::AnyValue;
pub use crate::acore::value::ErasedValue;
pub use crate::acore::value::InitHandler;
pub use crate::acore::value::InitializeValue;
pub use crate::acore::value::RawValParser;
pub use crate::acore::value::StoreHandler;
pub use crate::acore::value::ValAccessor;
pub use crate::acore::value::ValInitializer;
pub use crate::acore::value::ValStorer;
pub use crate::acore::value::ValValidator;
pub use crate::acore::value::ValidatorHandler;

use crate::ctx::Ctx;
use crate::Error;

/// A special option value, can stop the policy, using for implement `--`.
///
/// # Example
/// ```
/// use aopt::prelude::*;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///
///     let mut parser = AFwdParser::default();
///
///     parser.add_opt("stop".infer::<aopt::value::Stop>())?;
///
///     // -w will processed, it is set before `--`
///     parser.add_opt("-w=i")?;
///
///     // -o will not processed, it is set after `--`
///     parser.add_opt("-o=s")?;
///
///     // fo will processed, it is not an option
///     parser.add_opt("foo=p@1")?;
///
///     parser.parse(Args::from(["app", "-w=42", "--", "-o", "val", "foo"]))?;
///
///     assert_eq!(parser.find_val::<i64>("-w")?, &42);
///     assert!(parser.find_val::<String>("-o").is_err());
///     assert_eq!(parser.find_val::<bool>("foo")?, &true);
///     Ok(())
/// }
/// ```
///
/// ```plaintext
/// POSIX.1-2017
///
/// 12.2 Utility Syntax Guidelines
///
/// Guideline 10:
///
/// The first -- argument that is not an option-argument should be accepted as a delimiter indicating the end of options.
/// Any following arguments should be treated as operands, even if they begin with the '-' character.
/// ```
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Stop;

impl RawValParser for Stop {
    type Error = Error;

    fn parse(raw: Option<&OsStr>, ctx: &Ctx) -> Result<Self, Self::Error> {
        const STOP: &str = "--";

        if ctx.name()?.map(|v| v.as_ref()) == Some(STOP) {
            ctx.set_policy_act(crate::parser::Action::Stop);
            Ok(Stop)
        } else {
            Err(Error::sp_rawval(raw, "except `--` for Stop").with_uid(ctx.uid()?))
        }
    }
}

/// A special option value, using for implement `-`.
///
/// # Example
/// ```
/// use aopt::prelude::*;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///
///     let mut parser = AFwdParser::default();
///
///     parser.set_strict(true);
///     parser.add_opt("stdin=b".infer::<std::io::Stdin>())?;
///
///     // -w will processed, it is set before `--`
///     parser.add_opt("-w=i")?;
///
///     // -o will not processed, it is set after `--`
///     parser.add_opt("-o=s")?;
///
///     // fo will processed, it is not an option
///     parser.add_opt("foo=p@1")?;
///
///     parser.parse(Args::from(
///         ["app", "-w=42", "-", "foo"].into_iter(),
///     ))?;
///
///     assert_eq!(parser.find_val::<i64>("-w")?, &42);
///     assert!(parser.find_val::<std::io::Stdin>("-").is_ok());
///     assert_eq!(parser.find_val::<bool>("foo")?, &true);
///     Ok(())
/// }
/// ```
pub enum DocTestForRawValParserStdin {}
