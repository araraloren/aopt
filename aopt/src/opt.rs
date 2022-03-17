mod help;
mod index;
mod parser;
mod style;
mod value;

pub mod nonopt;
pub mod opt;

use std::fmt::Debug;
use ustr::Ustr;

use crate::err::Result;
use crate::uid::Uid;

cfg_if::cfg_if! {
    if #[cfg(feature = "sync")] {
        mod callback_sync;
        pub use self::callback_sync::Callback as OptCallback;
        pub use self::callback_sync::CallbackType;
        pub use self::callback_sync::MainFn;
        pub use self::callback_sync::MainFnMut;
        pub use self::callback_sync::OptFn;
        pub use self::callback_sync::OptFnMut;
        pub use self::callback_sync::PosFn;
        pub use self::callback_sync::PosFnMut;
        pub use self::callback_sync::SimpleMainFn;
        pub use self::callback_sync::SimpleMainFnMut;
        pub use self::callback_sync::SimpleOptFn;
        pub use self::callback_sync::SimpleOptFnMut;
        pub use self::callback_sync::SimplePosFn;
        pub use self::callback_sync::SimplePosFnMut;
    }
    else {
        mod callback;
        pub use self::callback::Callback as OptCallback;
        pub use self::callback::CallbackType;
        pub use self::callback::MainFn;
        pub use self::callback::MainFnMut;
        pub use self::callback::OptFn;
        pub use self::callback::OptFnMut;
        pub use self::callback::PosFn;
        pub use self::callback::PosFnMut;
        pub use self::callback::SimpleMainFn;
        pub use self::callback::SimpleMainFnMut;
        pub use self::callback::SimpleOptFn;
        pub use self::callback::SimpleOptFnMut;
        pub use self::callback::SimplePosFn;
        pub use self::callback::SimplePosFnMut;
    }
}

pub use self::help::create_help_hint;
pub use self::help::HelpInfo;
pub use self::index::Index as OptIndex;
pub use self::nonopt::CmdCreator;
pub use self::nonopt::MainCreator;
pub use self::nonopt::NonOpt;
pub use self::nonopt::PosCreator;
pub use self::opt::ArrayCreator;
pub use self::opt::BoolCreator;
pub use self::opt::FltCreator;
pub use self::opt::IntCreator;
pub use self::opt::StrCreator;
pub use self::opt::UintCreator;
pub use self::parser::parse_option_str;
pub use self::parser::DataKeeper;
pub use self::style::Style;
pub use self::value::CloneHelper;
pub use self::value::Value as OptValue;

/// The Type trait of option.
pub trait Type {
    /// Get the unique type name string of option type.
    fn get_type_name(&self) -> Ustr;

    /// Indicate if the option support deactivate style such as `--/boolean`.
    /// In defult is false.
    fn is_deactivate_style(&self) -> bool {
        false
    }

    /// Check if the option type support given style.
    fn match_style(&self, style: style::Style) -> bool;

    /// It will be called by [`Parser`](crate::parser::Parser) check the option validity.
    fn check(&self) -> Result<()>;

    fn as_any(&self) -> &dyn std::any::Any;
}

/// The Identifier trait of option.
pub trait Identifier {
    /// Get the unique identifier of current option.
    fn get_uid(&self) -> Uid;

    /// Set the unique identifier of current option.
    fn set_uid(&mut self, uid: Uid);
}

/// The Callback trait of option.
pub trait Callback {
    /// Check if we need invoke the callback of current option.
    fn is_need_invoke(&self) -> bool;

    /// The [`Context`](crate::ctx::Context) will set the value to true if user set an invalid value.
    fn set_invoke(&mut self, invoke: bool);

    /// Check if the option support given callback type.
    fn is_accept_callback_type(&self, callback_type: CallbackType) -> bool;

    /// Set the callback return value to option.
    fn set_callback_ret(&mut self, ret: Option<OptValue>) -> Result<()>;
}

/// The Name trait of option.
pub trait Name {
    /// Get the name of current option.
    fn get_name(&self) -> Ustr;

    /// Get the prefix of current option.
    fn get_prefix(&self) -> Ustr;

    /// Set the name of current option.
    fn set_name(&mut self, string: Ustr);

    /// Set the prefix of current option.
    fn set_prefix(&mut self, string: Ustr);

    /// Check if the option matched given name.
    fn match_name(&self, name: Ustr) -> bool;

    /// Check if the option matched given prefix.
    fn match_prefix(&self, prefix: Ustr) -> bool;
}

/// The Alias trait of option.
pub trait Alias {
    /// Get all the alias of current option.
    fn get_alias(&self) -> Option<&Vec<(Ustr, Ustr)>>;

    /// Add an alias to current option.
    fn add_alias(&mut self, prefix: Ustr, name: Ustr);

    /// Remove an alias of current option.
    fn rem_alias(&mut self, prefix: Ustr, name: Ustr);

    /// Check if any alias of the option matched given prefix and name.
    fn match_alias(&self, prefix: Ustr, name: Ustr) -> bool;
}

/// The Optional trait of option.
pub trait Optional {
    /// Get if the option is optional.
    fn get_optional(&self) -> bool;

    /// Set if the option is optional.
    fn set_optional(&mut self, optional: bool);

    /// Check if the option matched given optional value.
    fn match_optional(&self, optional: bool) -> bool;
}

/// The Value trait of option.
pub trait Value {
    /// Get value reference of current option.
    fn get_value(&self) -> &OptValue;

    /// Get mutable value reference of current option.
    fn get_value_mut(&mut self) -> &mut OptValue;

    /// Get default value reference of current option.
    fn get_default_value(&self) -> &OptValue;

    /// Set value of current option.
    fn set_value(&mut self, value: OptValue);

    /// Get default value of current option.
    fn set_default_value(&mut self, value: OptValue);

    /// Parse command line item and return an [`OptValue`].
    fn parse_value(&self, string: Ustr, disable: bool, index: u64) -> Result<OptValue>;

    /// Check if the option has a valid value.
    fn has_value(&self) -> bool;

    /// Reset the value to default value.
    fn reset_value(&mut self);
}

/// The Index trait of option.
pub trait Index {
    /// Get the index of current option.
    fn get_index(&self) -> Option<&OptIndex>;

    /// Set the index of current option.
    fn set_index(&mut self, index: OptIndex);

    /// Check if current option matched given [`NonOpt`](crate::opt::NonOpt) position.
    fn match_index(&self, total: u64, current: u64) -> bool;
}

/// The Help trait of option.
pub trait Help {
    /// Set the hint of current option.
    fn set_hint(&mut self, hint: Ustr);

    /// Set the help message of current option.
    fn set_help(&mut self, help: Ustr);

    /// Get the hint of current option.
    fn get_hint(&self) -> Ustr {
        self.get_help_info().get_hint()
    }

    /// Get the help message of current option.
    fn get_help(&self) -> Ustr {
        self.get_help_info().get_help()
    }

    /// Get help information of current option.
    fn get_help_info(&self) -> &HelpInfo;
}

cfg_if::cfg_if! {
    if #[cfg(feature = "sync")] {
        /// The option trait.
        pub trait Opt:
        Type + Identifier + Name + Callback + Alias + Optional + Value + Index + Help + Debug + Send + Sync
        { }
    }
    else {
        /// The option trait.
        pub trait Opt:
        Type + Identifier + Name + Callback + Alias + Optional + Value + Index + Help + Debug
        { }
    }
}

/// Create a [`OptCallback::Main`](crate::opt::OptCallback::Main)(Box<[`SimpleMainFn`](crate::opt::SimpleMainFn)>) from given block.
///
/// ## Example
///
/// ```ignore
/// // block type is `Fn(Uid, &S, &[&str], OptValue) -> Result<Option<OptValue>>`
/// simple_main_cb!(|uid, set, args, value| { Ok(Some(value)) });
/// ```
#[macro_export]
macro_rules! simple_main_cb {
    ($block:expr) => {
        OptCallback::Main(Box::new(SimpleMainFn::new($block)))
    };
}

/// Create a [`OptCallback::MainMut`](crate::opt::OptCallback::MainMut)(Box<[`SimpleMainFnMut`](crate::opt::SimpleMainFnMut)>) from given block.
///
/// ## Example
///
/// ```ignore
/// // block type is `FnMut(Uid, &mut S, &[&str], OptValue) -> Result<Option<OptValue>>`
/// simple_main_mut_cb!(|uid, set, args, value| { Ok(Some(value)) });
/// ```
#[macro_export]
macro_rules! simple_main_mut_cb {
    ($block:expr) => {
        OptCallback::MainMut(Box::new(SimpleMainFnMut::new($block)))
    };
}

/// Create a [`OptCallback::Pos`](crate::opt::OptCallback::Pos)(Box<[`SimplePosFn`](crate::opt::SimplePosFn)>) from given block.
///
/// ## Example
///
/// ```ignore
/// // block type is `Fn(Uid, &S, &str, u64, OptValue) -> Result<Option<OptValue>>`
/// simple_pos_cb!(|uid, set, arg, noa_i, value| { Ok(Some(value)) });
/// ```
#[macro_export]
macro_rules! simple_pos_cb {
    ($block:expr) => {
        OptCallback::Pos(Box::new(SimplePosFn::new($block)))
    };
}

/// Create a [`OptCallback::PosMut`](crate::opt::OptCallback::PosMut)(Box<[`SimplePosFnMut`](crate::opt::SimplePosFnMut)>) from given block.
///
/// ## Example
///
/// ```ignore
/// // block type is `FnMut(Uid, &mut S, &str, u64, OptValue) -> Result<Option<OptValue>>`
/// simple_pos_mut_cb!(|uid, set, arg, noa_i, value| { Ok(Some(value)) });
/// ```
#[macro_export]
macro_rules! simple_pos_mut_cb {
    ($block:expr) => {
        OptCallback::PosMut(Box::new(SimplePosFnMut::new($block)))
    };
}

/// Create a [`OptCallback::Opt`](crate::opt::OptCallback::Opt)(Box<[`SimpleOptFn`](crate::opt::SimpleOptFn)>) from given block.
///
/// ## Example
///
/// ```ignore
/// // block type is `Fn(Uid, &S, OptValue) -> Result<Option<OptValue>>`
/// simple_opt_cb!(|uid, set, value| { Ok(Some(value)) });
/// ```
#[macro_export]
macro_rules! simple_opt_cb {
    ($block:expr) => {
        OptCallback::Opt(Box::new(SimpleOptFn::new($block)))
    };
}

/// Create a [`OptCallback::OptMut`](crate::opt::OptCallback::OptMut)(Box<[`SimpleOptFnMut`](crate::opt::SimpleOptFnMut)>) from given block.
///
/// ## Example
///
/// ```ignore
/// // block type is `FnMut(Uid, &mut S, OptValue) -> Result<Option<OptValue>>`
/// simple_opt_mut_cb!(|uid, set, value| { Ok(Some(value)) });
/// ```
#[macro_export]
macro_rules! simple_opt_mut_cb {
    ($block:expr) => {
        OptCallback::OptMut(Box::new(SimpleOptFnMut::new($block)))
    };
}
