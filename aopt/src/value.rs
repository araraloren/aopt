pub(crate) mod accessor;
pub(crate) mod infer;
pub(crate) mod initializer;
pub(crate) mod parser;
pub(crate) mod storer;
pub(crate) mod validator;

use std::any::type_name;
use std::fmt::Debug;

pub use self::accessor::ValAccessor;
pub use self::infer::Infer;
pub use self::infer::Placeholder;
pub use self::initializer::InitHandler;
pub use self::initializer::InitializeValue;
pub use self::initializer::ValInitializer;
pub use self::parser::raw2str;
pub use self::parser::RawValParser;
pub use self::storer::StoreHandler;
pub use self::storer::ValStorer;
pub use self::validator::ValValidator;
pub use self::validator::ValidatorHandler;

use crate::ctx::Ctx;
use crate::map::AnyMap;
use crate::map::Entry;
use crate::map::ErasedTy;
use crate::opt::Action;
use crate::raise_error;
use crate::Error;
use crate::RawVal;

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
///     parser.set_strict(true);
///     parser.add_opt("stop=b".infer::<aopt::value::Stop>())?;
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
///     parser.parse(ARef::new(Args::from(
///         ["app", "-w=42", "--", "-o", "val", "foo"].into_iter(),
///     )))?;
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

pub trait ErasedValue {
    fn initialize(&mut self) -> Result<(), Error>;

    fn store(&mut self, raw: Option<&RawVal>, ctx: &Ctx, act: &Action) -> Result<(), Error>;

    fn store_act<U: ErasedTy>(&mut self, val: U, ctx: &Ctx, act: &Action) -> Result<(), Error>;

    fn val<U: ErasedTy>(&self) -> Result<&U, Error>;

    fn val_mut<U: ErasedTy>(&mut self) -> Result<&mut U, Error>;

    fn vals<U: ErasedTy>(&self) -> Result<&Vec<U>, Error>;

    fn vals_mut<U: ErasedTy>(&mut self) -> Result<&mut Vec<U>, Error>;

    fn rawval(&self) -> Result<&RawVal, Error>;

    fn rawval_mut(&mut self) -> Result<&mut RawVal, Error>;

    fn rawvals(&self) -> Result<&Vec<RawVal>, Error>;

    fn rawvals_mut(&mut self) -> Result<&mut Vec<RawVal>, Error>;
}

/// [`AnyValue`] can save values of any type. In internal it save the value into a vector of type T.
///
/// # Example
///
/// ```rust
/// # use aopt::prelude::*;
/// # use aopt::Error;
/// # use std::ops::SubAssign;
/// #
/// # fn main() -> Result<(), Error> {
/// let mut value = AnyValue::new();
///
/// assert_eq!(value.contain_type::<i32>(), false);
/// assert_eq!(value.set(vec![42]), None);
/// assert_eq!(value.contain_type::<i32>(), true);
///
/// assert_eq!(value.val::<i32>()?, &42);
/// assert_eq!(value.push(256).val::<i32>()?, &256);
/// value.val_mut::<i32>()?.sub_assign(128);
/// assert_eq!(value.val::<i32>()?, &128);
///
/// assert_eq!(value.vals::<i32>()?, &vec![42, 128]);
/// assert_eq!(value.pop::<i32>(), Some(128));
/// assert_eq!(value.vals::<i32>()?, &vec![42]);
///
/// value.entry::<u64>().or_insert(vec![9, 0, 2, 5]);
/// assert_eq!(value.entry::<u64>().or_default().pop(), Some(5));
///
/// value.vals_mut::<i32>()?.pop();
/// assert_eq!(value.vals::<i32>()?.len(), 0);
///
/// assert_eq!(value.remove::<u64>(), Some(vec![9, 0, 2]));
/// assert_eq!(value.contain_type::<u64>(), false);
/// #
/// # Ok(())
/// # }
/// ```
#[derive(Default)]
pub struct AnyValue(AnyMap);

impl Debug for AnyValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnyValue").field("inner", &self.0).finish()
    }
}

impl AnyValue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    pub fn contain_type<T: ErasedTy>(&self) -> bool {
        self.0.contain::<Vec<T>>()
    }

    fn inner<T: ErasedTy>(&self) -> Option<&Vec<T>> {
        self.0.value::<Vec<T>>()
    }

    fn inner_mut<T: ErasedTy>(&mut self) -> Option<&mut Vec<T>> {
        self.0.value_mut::<Vec<T>>()
    }

    pub fn pop<T: ErasedTy>(&mut self) -> Option<T> {
        self.inner_mut().and_then(|v| v.pop())
    }

    pub fn entry<T: ErasedTy>(&mut self) -> Entry<'_, Vec<T>> {
        self.0.entry::<Vec<T>>()
    }

    pub fn push<T: ErasedTy>(&mut self, val: T) -> &mut Self {
        self.entry::<T>().or_default().push(val);
        self
    }

    pub fn set<T: ErasedTy>(&mut self, vals: Vec<T>) -> Option<Vec<T>> {
        let ret = self.remove();
        self.entry().or_insert(vals);
        ret
    }

    pub fn remove<T: ErasedTy>(&mut self) -> Option<Vec<T>> {
        self.0.remove::<Vec<T>>()
    }

    /// Get the last value reference of type T.
    pub fn val<T: ErasedTy>(&self) -> Result<&T, Error> {
        self.inner().and_then(|v| v.last()).ok_or_else(|| {
            raise_error!(
                "Can not find value for type `{:?}` in ErasedVal(val)",
                type_name::<T>()
            )
        })
    }

    /// Get the last value mutable reference of type T.
    pub fn val_mut<T: ErasedTy>(&mut self) -> Result<&mut T, Error> {
        self.inner_mut().and_then(|v| v.last_mut()).ok_or_else(|| {
            raise_error!(
                "Can not find value for type `{:?}` in ErasedVal(val_mut)",
                type_name::<T>()
            )
        })
    }

    /// Get the values of type T.
    pub fn vals<T: ErasedTy>(&self) -> Result<&Vec<T>, Error> {
        self.inner().ok_or_else(|| {
            raise_error!(
                "Can not find value for type `{:?}` in ErasedVal(vals)",
                type_name::<T>()
            )
        })
    }

    /// Get the values of type T.
    pub fn vals_mut<T: ErasedTy>(&mut self) -> Result<&mut Vec<T>, Error> {
        self.inner_mut().ok_or_else(|| {
            raise_error!(
                "Can not find value for type `{:?}` in ErasedVal(vals_mut)",
                type_name::<T>()
            )
        })
    }
}
