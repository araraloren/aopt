
use std::{borrow::Cow, ops::{Deref, DerefMut}};
use std::cmp::{Eq, PartialEq};

/// All the inner struct will using this type
/// hold the string value .
/// 
/// ```no_run
/// use getopt_rs::str::Str;
/// 
/// fn cow_string_example() {
///    #[derive(Debug)]
///    struct Holder<'a> {
///        data: Str<'a>,
///    }
///
///    impl<'a, 'b: 'a> Holder<'a> {
///        pub fn new(data: &'b str) -> Self {
///            Self {
///                data: Str::borrowed(data),
///            }
///        }
///    }
///
///    let s = String::from("inner_data");
///
///    dbg!(Holder::new(s.as_ref()));
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Str<'a>(Cow<'a, str>);

impl<'a> Str<'a> {
    pub fn borrowed<T: Into<&'a str>>(s: T) -> Self {
        Self(Cow::Borrowed(s.into()))
    }

    pub fn owned<T: Into<String>>(s: T) -> Self {
        Self(Cow::Owned(s.into()))
    }
}

impl<'a> Default for Str<'a> {
    fn default() -> Self {
        Self(Cow::Borrowed(""))
    }
}

impl<'a> Deref for Str<'a> {
    type Target = Cow<'a, str>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for Str<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a> AsRef<str> for Str<'a> {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl<'a> PartialEq<str> for Str<'a> {
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl<'a, 'b> PartialEq<&'b str> for Str<'a> {
    fn eq(&self, other: &&'b str) -> bool {
        self.0 == *other
    }
}

impl<'a> PartialEq<String> for Str<'a> {
    fn eq(&self, other: &String) -> bool {
        self.0 == other.as_ref()
    }
}

impl<'a, 'b> PartialEq<&'b String> for Str<'a> {
    fn eq(&self, other: &&'b String) -> bool {
        self.0 == (*other).as_ref()
    }
}