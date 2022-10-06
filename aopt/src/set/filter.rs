use std::fmt::Debug;

use super::Prefixed;
use crate::opt::Config;
use crate::opt::ConfigValue;
use crate::opt::Creator;
use crate::opt::Information;
use crate::opt::Opt;
use crate::opt::OptIndex;
use crate::opt::OptParser;
use crate::set::OptSet;
use crate::Str;

/// Matching implementation for option and [`ConfigValue`].
pub trait FilterMatcher<T>
where
    T: Opt,
{
    fn match_opt(&self, opt: &T) -> bool;
}

impl<C, T> FilterMatcher<T> for C
where
    T: Opt,
    C: Config + ConfigValue,
{
    fn match_opt(&self, opt: &T) -> bool {
        let mut ret = true;

        if ret && self.has_deactivate_style() {
            ret = ret && (self.get_deactivate_style().unwrap() == opt.is_deactivate_style());
        }
        if ret && self.has_optional() {
            ret = ret && (self.get_optional().unwrap() == opt.get_optional());
        }
        if ret && self.has_type_name() {
            ret = ret && (self.get_type_name().unwrap() == opt.get_type_name());
        }
        if ret && self.has_prefix() {
            // don't call match prefix
            let mut matched = opt.get_prefix() == self.get_prefix();

            if !matched {
                let prefix = self.get_prefix().unwrap();

                if let Some(alias) = opt.get_alias().as_ref() {
                    for item in alias.iter() {
                        if item.0 == prefix {
                            matched = true;
                            break;
                        }
                    }
                }
            }
            ret = ret && matched;
        }
        if ret && self.has_name() {
            // don't call match name
            let name = self.get_name().unwrap();
            let mut matched = opt.get_name() == name;

            if !matched {
                if let Some(alias) = opt.get_alias().as_ref() {
                    for item in alias.iter() {
                        if item.1 == name {
                            matched = true;
                            break;
                        }
                    }
                }
            }
            ret = ret && matched;
        }
        if ret && self.has_index() {
            if let Some(index) = opt.get_index() {
                ret = ret && (self.get_index().unwrap() == index);
            }
        }
        ret
    }
}

/// Filter the option using given configurations.
pub struct Filter<'a, T, Parser, Ctor>
where
    T: Opt,
    Ctor: Creator<Opt = T>,
    Parser: OptParser,
    Ctor::Config: Config + ConfigValue,
{
    info: Ctor::Config,
    set: &'a OptSet<T, Parser, Ctor>,
}

impl<'a, T, Parser, Ctor> Debug for Filter<'a, T, Parser, Ctor>
where
    T: Opt,
    Ctor: Creator<Opt = T> + Debug,
    Parser: OptParser + Debug,
    Ctor::Config: Config + ConfigValue + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Filter")
            .field("info", &self.info)
            .field("set", &self.set)
            .finish()
    }
}

impl<'a, T, Parser, Ctor> Filter<'a, T, Parser, Ctor>
where
    T: Opt,
    Ctor: Creator<Opt = T>,
    Parser: OptParser + Prefixed,
    Parser::Output: Information,
    Ctor::Config: Config + ConfigValue + Default,
{
    pub fn new(set: &'a OptSet<T, Parser, Ctor>, info: Ctor::Config) -> Self {
        Self { set, info }
    }

    /// Set the option name of filter configuration.
    pub fn set_name<S: Into<Str>>(&mut self, name: S) -> &mut Self {
        self.info.set_name(name);
        self
    }

    /// Set the option prefix of filter configuration.
    pub fn set_prefix<S: Into<Str>>(&mut self, prefix: S) -> &mut Self {
        self.info.set_prefix(prefix);
        self
    }

    /// Set the option type name of filter configuration.
    pub fn set_type_name<S: Into<Str>>(&mut self, type_name: S) -> &mut Self {
        self.info.set_type_name(type_name);
        self
    }

    /// Set the option index of filter configuration.
    pub fn set_index(&mut self, index: OptIndex) -> &mut Self {
        self.info.set_index(index);
        self
    }

    /// Set the option optional of filter configuration.
    pub fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.info.set_optional(optional);
        self
    }

    /// Set the option deactivate style of filter configuration.
    pub fn set_deactivate_style(&mut self, deactivate_style: bool) -> &mut Self {
        self.info.set_deactivate_style(deactivate_style);
        self
    }

    /// Find the option by configuration, return None if not found.
    pub fn find(&self) -> Option<&'_ T> {
        self.set.iter().find(|opt| self.info.match_opt(*opt))
    }

    /// Find the option by configuration, return an iterator of `&T`.
    pub fn find_all(&self) -> impl Iterator<Item = &T> {
        self.set.iter().filter(|opt| self.info.match_opt(*opt))
    }
}

/// Filter the option using given configurations.
pub struct FilterMut<'a, T, Parser, Ctor>
where
    T: Opt,
    Ctor: Creator<Opt = T>,
    Parser: OptParser,
    Ctor::Config: Config + ConfigValue,
{
    info: Ctor::Config,
    set: &'a mut OptSet<T, Parser, Ctor>,
}

impl<'a, T, Parser, Ctor> Debug for FilterMut<'a, T, Parser, Ctor>
where
    T: Opt,
    Ctor: Creator<Opt = T> + Debug,
    Parser: OptParser + Debug,
    Ctor::Config: Config + ConfigValue + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FilterMut")
            .field("info", &self.info)
            .field("set", &self.set)
            .finish()
    }
}

impl<'a, T, Parser, Ctor> FilterMut<'a, T, Parser, Ctor>
where
    T: Opt,
    Ctor: Creator<Opt = T>,
    Parser: OptParser + Prefixed,
    Parser::Output: Information,
    Ctor::Config: Config + ConfigValue + Default,
{
    pub fn new(set: &'a mut OptSet<T, Parser, Ctor>, info: Ctor::Config) -> Self {
        Self { set, info }
    }

    /// Set the option name of filter configuration.
    pub fn set_name<S: Into<Str>>(&mut self, name: S) -> &mut Self {
        self.info.set_name(name);
        self
    }

    /// Set the option prefix of filter configuration.
    pub fn set_prefix<S: Into<Str>>(&mut self, prefix: S) -> &mut Self {
        self.info.set_prefix(prefix);
        self
    }

    /// Set the option type name of filter configuration.
    pub fn set_type_name<S: Into<Str>>(&mut self, type_name: S) -> &mut Self {
        self.info.set_type_name(type_name);
        self
    }

    /// Set the option index of filter configuration.
    pub fn set_index(&mut self, index: OptIndex) -> &mut Self {
        self.info.set_index(index);
        self
    }

    /// Set the option optional of filter configuration.
    pub fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.info.set_optional(optional);
        self
    }

    /// Set the option deactivate style of filter configuration.
    pub fn set_deactivate_style(&mut self, deactivate_style: bool) -> &mut Self {
        self.info.set_deactivate_style(deactivate_style);
        self
    }

    /// Find the option by configuration, return None if not found.
    pub fn find(&mut self) -> Option<&mut T> {
        self.set.iter_mut().find(|opt| self.info.match_opt(*opt))
    }

    /// Find the option by configuration, return an iterator of `&mut T`.
    pub fn find_all(&mut self) -> impl Iterator<Item = &mut T> {
        self.set.iter_mut().filter(|opt| self.info.match_opt(*opt))
    }
}
