use std::any::TypeId;
use std::fmt::Debug;

use crate::opt::ConfigValue;
use crate::opt::Index;
use crate::opt::Opt;
use crate::set::Ctor;
use crate::set::Set;
use crate::set::SetCfg;
use crate::set::SetOpt;

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
    C: ConfigValue,
{
    /// Check if option matched current option.
    fn mat_opt(&self, opt: &T) -> bool {
        let mut ret = true;

        if ret && self.has_force() {
            ret = ret && (self.force().unwrap() == opt.force());
        }
        if ret && self.has_type() {
            ret = ret && (self.r#type().unwrap() == opt.r#type());
        }
        if ret && self.has_name() {
            // don't call match name
            let name = self.name().unwrap();
            let mut matched = opt.name() == name;

            if !matched {
                if let Some(alias) = opt.alias().as_ref() {
                    for item in alias.iter() {
                        if item == name {
                            matched = true;
                            break;
                        }
                    }
                }
            }
            ret = ret && matched;
        }
        if ret && self.has_index() {
            if let Some(index) = opt.index() {
                ret = ret && (self.index().unwrap() == index);
            } else {
                ret = false;
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
    SetCfg<S>: ConfigValue,
{
    set: &'a S,
    info: SetCfg<S>,
}

impl<S> Debug for Filter<'_, S>
where
    S: Set + Debug,
    S::Ctor: Ctor,
    SetCfg<S>: ConfigValue + Debug,
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
    SetCfg<S>: ConfigValue,
{
    pub fn new(set: &'a S, info: SetCfg<S>) -> Self {
        Self { set, info }
    }

    /// Set the option name of filter configuration.
    pub fn set_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.info.set_name(name);
        self
    }

    /// Set the option type name of filter configuration.
    pub fn set_type<U: 'static>(&mut self) -> &mut Self {
        self.info.set_type::<U>();
        self
    }

    /// Set the option type name of filter configuration.
    pub fn set_type_id(&mut self, type_id: TypeId) -> &mut Self {
        self.info.set_type_id(type_id);
        self
    }

    /// Set the option index of filter configuration.
    pub fn set_index(&mut self, index: Index) -> &mut Self {
        self.info.set_index(index);
        self
    }

    /// Set the option optional of filter configuration.
    pub fn set_force(&mut self, force: bool) -> &mut Self {
        self.info.set_force(force);
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
    SetCfg<S>: ConfigValue,
{
    set: &'a mut S,
    info: SetCfg<S>,
}

impl<S> Debug for FilterMut<'_, S>
where
    S: Set + Debug,
    S::Ctor: Ctor,
    SetCfg<S>: ConfigValue + Debug,
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
    SetCfg<S>: ConfigValue,
{
    pub fn new(set: &'a mut S, info: SetCfg<S>) -> Self {
        Self { set, info }
    }

    /// Set the option name of filter configuration.
    pub fn set_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.info.set_name(name);
        self
    }

    /// Set the option type name of filter configuration.
    pub fn set_type<U: 'static>(&mut self) -> &mut Self {
        self.info.set_type::<U>();
        self
    }

    /// Set the option type name of filter configuration.
    pub fn set_type_id(&mut self, type_id: TypeId) -> &mut Self {
        self.info.set_type_id(type_id);
        self
    }

    /// Set the option index of filter configuration.
    pub fn set_index(&mut self, index: Index) -> &mut Self {
        self.info.set_index(index);
        self
    }

    /// Set the option optional of filter configuration.
    pub fn set_force(&mut self, force: bool) -> &mut Self {
        self.info.set_force(force);
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
