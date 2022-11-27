use std::fmt::Debug;

use crate::opt::Config;
use crate::opt::ConfigValue;
use crate::opt::Ctor;
use crate::opt::Index;
use crate::opt::Opt;
use crate::set::Set;
use crate::set::SetCfg;
use crate::set::SetOpt;
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
pub struct Filter<'a, S>
where
    S: Set,
    S::Ctor: Ctor,
    SetCfg<S>: Config + ConfigValue,
{
    set: &'a S,
    info: SetCfg<S>,
}

impl<'a, S> Debug for Filter<'a, S>
where
    S: Set + Debug,
    S::Ctor: Ctor,
    SetCfg<S>: Config + ConfigValue + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Filter")
            .field("set", &self.set)
            .field("info", &self.info)
            .finish()
    }
}

impl<'a, S> Filter<'a, S>
where
    S: Set,
    S::Ctor: Ctor,
    SetOpt<S>: Opt,
    SetCfg<S>: Config + ConfigValue,
{
    pub fn new(set: &'a S, info: SetCfg<S>) -> Self {
        Self { set, info }
    }

    /// Set the option name of filter configuration.
    pub fn set_name<T: Into<Str>>(&mut self, name: T) -> &mut Self {
        self.info.set_name(name);
        self
    }

    /// Set the option prefix of filter configuration.
    pub fn set_pre<T: Into<Str>>(&mut self, prefix: T) -> &mut Self {
        self.info.set_prefix(prefix);
        self
    }

    /// Set the option type name of filter configuration.
    pub fn set_ty<T: Into<Str>>(&mut self, type_name: T) -> &mut Self {
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
    pub fn find(&self) -> Option<&'_ SetOpt<S>> {
        self.set.iter().find(|v| self.info.mat_opt(*v))
    }

    /// Find the option by configuration, return a vector of `&T`.
    pub fn find_all(&self) -> impl Iterator<Item = &SetOpt<S>> {
        self.set.iter().filter(|v| self.info.mat_opt(*v))
    }
}

/// Filter the option using given configurations.
pub struct FilterMut<'a, S>
where
    S: Set,
    S::Ctor: Ctor,
    SetCfg<S>: Config + ConfigValue,
{
    set: &'a mut S,
    info: SetCfg<S>,
}

impl<'a, S> Debug for FilterMut<'a, S>
where
    S: Set + Debug,
    S::Ctor: Ctor,
    SetCfg<S>: Config + ConfigValue + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FilterMut")
            .field("info", &self.info)
            .field("set", &self.set)
            .finish()
    }
}

impl<'a, S> FilterMut<'a, S>
where
    S: Set,
    S::Ctor: Ctor,
    SetOpt<S>: Opt,
    SetCfg<S>: Config + ConfigValue,
{
    pub fn new(set: &'a mut S, info: SetCfg<S>) -> Self {
        Self { set, info }
    }

    /// Set the option name of filter configuration.
    pub fn set_name<T: Into<Str>>(&mut self, name: T) -> &mut Self {
        self.info.set_name(name);
        self
    }

    /// Set the option prefix of filter configuration.
    pub fn set_pre<T: Into<Str>>(&mut self, prefix: T) -> &mut Self {
        self.info.set_prefix(prefix);
        self
    }

    /// Set the option type name of filter configuration.
    pub fn set_ty<T: Into<Str>>(&mut self, type_name: T) -> &mut Self {
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
    pub fn find(&mut self) -> Option<&mut SetOpt<S>> {
        self.set.iter_mut().find(|v| self.info.mat_opt(*v))
    }

    /// Find the option by configuration, return an iterator of `&mut T`.
    pub fn find_all(&mut self) -> impl Iterator<Item = &mut SetOpt<S>> {
        self.set.iter_mut().filter(|v| self.info.mat_opt(*v))
    }
}
