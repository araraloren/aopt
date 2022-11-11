use std::fmt::Debug;

use crate::opt::Config;
use crate::opt::ConfigValue;
use crate::opt::Creator;
use crate::opt::Index;
use crate::opt::Information;
use crate::opt::Opt;
use crate::opt::OptParser;
use crate::set::OptSet;
use crate::set::Pre;
use crate::Str;

/// Matching implementation for option and [`ConfigValue`].
pub trait FilterMatcher<T>
where
    T: Opt,
{
    fn mat_opt(&self, opt: &T) -> bool;
}

impl<C, T> FilterMatcher<T> for C
where
    T: Opt,
    C: Config + ConfigValue,
{
    fn mat_opt(&self, opt: &T) -> bool {
        let mut ret = true;

        if ret && self.has_deactivate() {
            ret = ret && (self.deactivate().unwrap() == opt.is_deactivate());
        }
        if ret && self.has_optional() {
            ret = ret && (self.optional().unwrap() == opt.optional());
        }
        if ret && self.has_type() {
            ret = ret && (self.r#type().unwrap() == &opt.r#type());
        }
        if ret && self.has_prefix() {
            // don't call match prefix
            let mut matched = opt.prefix() == self.prefix();

            if !matched {
                let prefix = self.prefix().unwrap();

                if let Some(alias) = opt.alias().as_ref() {
                    for item in alias.iter() {
                        if &item.0 == prefix {
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
            let name = self.name().unwrap();
            let mut matched = opt.name() == name;

            if !matched {
                if let Some(alias) = opt.alias().as_ref() {
                    for item in alias.iter() {
                        if &item.1 == name {
                            matched = true;
                            break;
                        }
                    }
                }
            }
            ret = ret && matched;
        }
        if ret && self.has_idx() {
            if let Some(index) = opt.idx() {
                ret = ret && (self.idx().unwrap() == index);
            }
        }
        ret
    }
}

/// Filter the option using given configurations.
pub struct Filter<'a, Parser, Ctor>
where
    Ctor: Creator,
    Parser: OptParser,
    Ctor::Config: Config + ConfigValue,
{
    info: Ctor::Config,
    set: &'a OptSet<Parser, Ctor>,
}

impl<'a, Parser, Ctor> Debug for Filter<'a, Parser, Ctor>
where
    Ctor::Opt: Debug,
    Ctor: Creator + Debug,
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

impl<'a, Parser, Ctor> Filter<'a, Parser, Ctor>
where
    Ctor::Opt: Opt,
    Ctor: Creator,
    Parser: OptParser + Pre,
    Parser::Output: Information,
    Ctor::Config: Config + ConfigValue + Default,
{
    pub fn new(set: &'a OptSet<Parser, Ctor>, info: Ctor::Config) -> Self {
        Self { set, info }
    }

    /// Set the option name of filter configuration.
    pub fn set_name<S: Into<Str>>(&mut self, name: S) -> &mut Self {
        self.info.set_name(name);
        self
    }

    /// Set the option prefix of filter configuration.
    pub fn set_pre<S: Into<Str>>(&mut self, prefix: S) -> &mut Self {
        self.info.set_prefix(prefix);
        self
    }

    /// Set the option type name of filter configuration.
    pub fn set_ty<S: Into<Str>>(&mut self, type_name: S) -> &mut Self {
        self.info.set_type(type_name);
        self
    }

    /// Set the option index of filter configuration.
    pub fn set_idx(&mut self, index: Index) -> &mut Self {
        self.info.set_idx(index);
        self
    }

    /// Set the option optional of filter configuration.
    pub fn set_opt(&mut self, optional: bool) -> &mut Self {
        self.info.set_optional(optional);
        self
    }

    /// Set the option deactivate style of filter configuration.
    pub fn set_deact(&mut self, deactivate_style: bool) -> &mut Self {
        self.info.set_deactivate(deactivate_style);
        self
    }

    /// Find the option by configuration, return None if not found.
    pub fn find(&self) -> Option<&'_ Ctor::Opt> {
        self.set.iter().find(|opt| self.info.mat_opt(*opt))
    }

    /// Find the option by configuration, return an iterator of `&T`.
    pub fn find_all(&self) -> impl Iterator<Item = &Ctor::Opt> {
        self.set.iter().filter(|opt| self.info.mat_opt(*opt))
    }
}

/// Filter the option using given configurations.
pub struct FilterMut<'a, Parser, Ctor>
where
    Ctor: Creator,
    Parser: OptParser,
    Ctor::Config: Config + ConfigValue,
{
    info: Ctor::Config,
    set: &'a mut OptSet<Parser, Ctor>,
}

impl<'a, Parser, Ctor> Debug for FilterMut<'a, Parser, Ctor>
where
    Ctor::Opt: Opt,
    Ctor: Creator + Debug,
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

impl<'a, Parser, Ctor> FilterMut<'a, Parser, Ctor>
where
    Ctor::Opt: Opt,
    Ctor: Creator,
    Parser: OptParser + Pre,
    Parser::Output: Information,
    Ctor::Config: Config + ConfigValue + Default,
{
    pub fn new(set: &'a mut OptSet<Parser, Ctor>, info: Ctor::Config) -> Self {
        Self { set, info }
    }

    /// Set the option name of filter configuration.
    pub fn set_name<S: Into<Str>>(&mut self, name: S) -> &mut Self {
        self.info.set_name(name);
        self
    }

    /// Set the option prefix of filter configuration.
    pub fn set_pre<S: Into<Str>>(&mut self, prefix: S) -> &mut Self {
        self.info.set_prefix(prefix);
        self
    }

    /// Set the option type name of filter configuration.
    pub fn set_ty<S: Into<Str>>(&mut self, type_name: S) -> &mut Self {
        self.info.set_type(type_name);
        self
    }

    /// Set the option index of filter configuration.
    pub fn set_idx(&mut self, index: Index) -> &mut Self {
        self.info.set_idx(index);
        self
    }

    /// Set the option optional of filter configuration.
    pub fn set_opt(&mut self, optional: bool) -> &mut Self {
        self.info.set_optional(optional);
        self
    }

    /// Set the option deactivate style of filter configuration.
    pub fn set_deact(&mut self, deactivate_style: bool) -> &mut Self {
        self.info.set_deactivate(deactivate_style);
        self
    }

    /// Find the option by configuration, return None if not found.
    pub fn find(&mut self) -> Option<&mut Ctor::Opt> {
        self.set.iter_mut().find(|opt| self.info.mat_opt(*opt))
    }

    /// Find the option by configuration, return an iterator of `&mut T`.
    pub fn find_all(&mut self) -> impl Iterator<Item = &mut Ctor::Opt> {
        self.set.iter_mut().filter(|opt| self.info.mat_opt(*opt))
    }
}
