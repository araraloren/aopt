use std::fmt::Debug;

use crate::opt::ConfigValue;
use crate::opt::Opt;
use crate::set::Ctor;
use crate::trace;
use crate::Error;

#[cfg(feature = "sync")]
mod __creator {
    use super::*;

    pub struct Creator<O, C, E: Into<Error>> {
        pub(crate) cid: Cid,

        pub(crate) callback: Box<dyn FnMut(C) -> Result<O, E> + Send + Sync + 'static>,
    }

    impl<O: Opt, C, E: Into<Error>> Debug for Creator<O, C, E> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Creator")
                .field("cid", &self.cid)
                .field("callback", &"{...}")
                .finish()
        }
    }

    impl<O: Opt, C, E: Into<Error>> Creator<O, C, E> {
        pub fn new(
            cid: Cid,
            callback: impl FnMut(C) -> Result<O, E> + Send + Sync + 'static,
        ) -> Self {
            Self {
                cid,
                callback: Box::new(callback),
            }
        }
    }
}

#[cfg(not(feature = "sync"))]
mod __creator {
    use super::*;

    pub struct Creator<O, C, E: Into<Error>> {
        pub(crate) cid: Cid,

        pub(crate) callback: Box<dyn FnMut(C) -> Result<O, E> + 'static>,
    }

    impl<O: Opt, C, E: Into<Error>> Debug for Creator<O, C, E> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("Creator")
                .field("cid", &self.cid)
                .field("callback", &"{...}")
                .finish()
        }
    }

    impl<O: Opt, C, E: Into<Error>> Creator<O, C, E> {
        pub fn new(cid: Cid, callback: impl FnMut(C) -> Result<O, E> + 'static) -> Self {
            Self {
                cid,
                callback: Box::new(callback),
            }
        }
    }
}

pub use __creator::Creator;

impl<O: Opt, C, E: Into<Error>> Ctor for Creator<O, C, E> {
    type Opt = O;

    type Config = C;

    type Error = E;

    fn cid(&self) -> &Cid {
        &self.cid
    }

    fn new_with(&mut self, config: Self::Config) -> Result<Self::Opt, Self::Error> {
        (self.callback)(config)
    }
}

pub(crate) const CID_INT_SHORT: &str = "i";
pub(crate) const CID_INT_LONG: &str = "int";
pub(crate) const CID_INT_TYPE: &str = "i64";
pub(crate) const CID_BOOL_SHORT: &str = "b";
pub(crate) const CID_BOOL_LONG: &str = "boolean";
pub(crate) const CID_BOOL_TYPE: &str = "bool";
pub(crate) const CID_UINT_SHORT: &str = "u";
pub(crate) const CID_UINT_LONG: &str = "uint";
pub(crate) const CID_UINT_TYPE: &str = "u64";
pub(crate) const CID_STR_SHORT: &str = "s";
pub(crate) const CID_STR_LONG: &str = "str";
pub(crate) const CID_STR_TYPE: &str = "string";
pub(crate) const CID_FLT_SHORT: &str = "f";
pub(crate) const CID_FLT_LONG: &str = "flt";
pub(crate) const CID_FLT_TYPE: &str = "f64";
pub(crate) const CID_CMD_SHORT: &str = "c";
pub(crate) const CID_CMD_LONG: &str = "cmd";
pub(crate) const CID_POS_SHORT: &str = "p";
pub(crate) const CID_POS_LONG: &str = "pos";
pub(crate) const CID_MAIN_SHORT: &str = "m";
pub(crate) const CID_MAIN_LONG: &str = "main";
pub(crate) const CID_ANY_SHORT: &str = "a";
pub(crate) const CID_ANY_LONG: &str = "any";
pub(crate) const CID_RAW_SHORT: &str = "r";
pub(crate) const CID_RAW_LONG: &str = "raw";
pub(crate) const CID_FALLBACK: &str = "fallback";

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Cid {
    /// Create names: `i`, `int`, `i64`
    Int,

    /// Create names: `s`, `str`, `string`
    Str,

    /// Create names: `f`, `flt`, `f64`
    Flt,

    /// Create names: `u`, `uint`, `u64`
    Uint,

    /// Create names: `b`, `boolean`, `bool`
    Bool,

    /// Create names: `c`, `cmd`
    Cmd,

    /// Create names: `p`, `pos`
    Pos,

    /// Create names: `m`, `main`
    Main,

    /// Create names: `a`, `any`
    Any,

    /// Create names: `r`, `raw`
    Raw,

    /// Create names: `fallback`
    Fallback,

    Name(String),
}

impl Cid {
    pub fn is_suit<S: AsRef<str>>(&self, s: &S) -> bool {
        let s = s.as_ref();

        match self {
            Cid::Int => matches!(s, CID_INT_SHORT | CID_INT_LONG | CID_INT_TYPE),
            Cid::Str => matches!(s, CID_STR_SHORT | CID_STR_LONG | CID_STR_TYPE),
            Cid::Flt => matches!(s, CID_FLT_SHORT | CID_FLT_LONG | CID_FLT_TYPE),
            Cid::Uint => matches!(s, CID_UINT_SHORT | CID_UINT_LONG | CID_UINT_TYPE),
            Cid::Bool => matches!(s, CID_BOOL_SHORT | CID_BOOL_LONG | CID_BOOL_TYPE),
            Cid::Cmd => matches!(s, CID_CMD_SHORT | CID_CMD_LONG),
            Cid::Pos => matches!(s, CID_POS_SHORT | CID_POS_LONG),
            Cid::Main => matches!(s, CID_MAIN_SHORT | CID_MAIN_LONG),
            Cid::Any => matches!(s, CID_ANY_SHORT | CID_ANY_LONG),
            Cid::Raw => matches!(s, CID_RAW_SHORT | CID_RAW_LONG),
            Cid::Fallback => matches!(s, CID_FALLBACK),
            Cid::Name(name) => s == name.as_str(),
        }
    }
}

impl From<String> for Cid {
    fn from(value: String) -> Self {
        let s = value.as_str();

        match s {
            CID_INT_SHORT | CID_INT_LONG | CID_INT_TYPE => Cid::Int,
            CID_STR_SHORT | CID_STR_LONG | CID_STR_TYPE => Cid::Str,
            CID_FLT_SHORT | CID_FLT_LONG | CID_FLT_TYPE => Cid::Flt,
            CID_UINT_SHORT | CID_UINT_LONG | CID_UINT_TYPE => Cid::Uint,
            CID_BOOL_SHORT | CID_BOOL_LONG | CID_BOOL_TYPE => Cid::Bool,
            CID_CMD_SHORT | CID_CMD_LONG => Cid::Cmd,
            CID_POS_SHORT | CID_POS_LONG => Cid::Pos,
            CID_MAIN_SHORT | CID_MAIN_LONG => Cid::Main,
            CID_ANY_SHORT | CID_ANY_LONG => Cid::Any,
            CID_RAW_SHORT | CID_RAW_LONG => Cid::Raw,
            CID_FALLBACK => Cid::Fallback,
            _ => Cid::Name(value),
        }
    }
}

impl From<&String> for Cid {
    fn from(value: &String) -> Self {
        Cid::from(value.clone())
    }
}

impl From<&str> for Cid {
    fn from(s: &str) -> Self {
        match s {
            CID_INT_SHORT | CID_INT_LONG | CID_INT_TYPE => Cid::Int,
            CID_STR_SHORT | CID_STR_LONG | CID_STR_TYPE => Cid::Str,
            CID_FLT_SHORT | CID_FLT_LONG | CID_FLT_TYPE => Cid::Flt,
            CID_UINT_SHORT | CID_UINT_LONG | CID_UINT_TYPE => Cid::Uint,
            CID_BOOL_SHORT | CID_BOOL_LONG | CID_BOOL_TYPE => Cid::Bool,
            CID_CMD_SHORT | CID_CMD_LONG => Cid::Cmd,
            CID_POS_SHORT | CID_POS_LONG => Cid::Pos,
            CID_MAIN_SHORT | CID_MAIN_LONG => Cid::Main,
            CID_ANY_SHORT | CID_ANY_LONG => Cid::Any,
            CID_RAW_SHORT | CID_RAW_LONG => Cid::Raw,
            CID_FALLBACK => Cid::Fallback,
            s => Cid::Name(String::from(s)),
        }
    }
}

impl<T: Opt + TryFrom<C, Error: Into<Error>>, C: ConfigValue + Debug> Creator<T, C, Error> {
    pub fn fallback() -> Self {
        Self::new(Cid::Fallback, move |config: C| {
            trace!("in fallback, construct option with config {:?}", &config);

            T::try_from(config).map_err(Into::into)
        })
    }

    pub fn new_type_ctor(ctor: Cid) -> Self {
        if ctor == Cid::Fallback {
            return Self::fallback();
        }
        Self::new(ctor, move |config: C| {
            trace!("construct option with config {:?}", &config);

            T::try_from(config).map_err(Into::into)
        })
    }
}

impl<T: Opt + TryFrom<C, Error: Into<Error>>, C: ConfigValue + Debug> From<Cid>
    for Creator<T, C, Error>
{
    fn from(value: Cid) -> Self {
        Self::new_type_ctor(value)
    }
}

/// Return an array of creators:
///
/// * [`Fallback`](Cid::Fallback)
/// * [`Int`](Cid::Int)
/// * [`Bool`](Cid::Bool)
/// * [`Flt`](Cid::Flt)
/// * [`AStr`](Cid::AStr)
/// * [`Uint`](Cid::Uint)
/// * [`Cmd`](Cid::Cmd)
/// * [`Pos`](Cid::Pos)
/// * [`Main`](Cid::Main)
/// * [`Any`](Cid::Any)
/// * [`Raw`](Cid::Raw)
#[macro_export]
macro_rules! ctors {
    ($type:ident) => {
        $crate::ctors!(
            $type,
            fallback,
            int,
            bool,
            flt,
            str,
            uint,
            cmd,
            pos,
            main,
            any,
            raw
        )
    };
    ($type:ident, $($creator:ident),+) => {
        {
            let mut ret = $crate::HashMap::new();

            $(
                ret.insert(
                    $crate::opt::Cid::from( stringify!($creator) ),
                    <$type>::from(
                        $crate::opt::Cid::from( stringify!($creator) )
                    )
                );
            )+
            ret
        }
    };
}
