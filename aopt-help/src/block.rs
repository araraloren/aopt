use std::borrow::Cow;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::format::HelpDisplay;
use crate::format::HelpPolicy;

#[derive(Debug, Default, Clone)]
pub struct Block<'a, T> {
    name: Cow<'a, str>,

    hint: Cow<'a, str>,

    help: Cow<'a, str>,

    foot: Cow<'a, str>,

    head: Cow<'a, str>,

    stores: Vec<T>,
}

impl<'a, T> Block<'a, T> {
    pub fn new<S: Into<Cow<'a, str>>>(name: S, hint: S, help: S, head: S, foot: S) -> Self {
        Self {
            name: name.into(),
            hint: hint.into(),
            help: help.into(),
            foot: foot.into(),
            head: head.into(),
            stores: vec![],
        }
    }

    pub fn name(&self) -> Cow<'a, str> {
        self.name.clone()
    }

    pub fn hint(&self) -> Cow<'a, str> {
        self.hint.clone()
    }

    pub fn help(&self) -> Cow<'a, str> {
        self.help.clone()
    }

    pub fn foot(&self) -> Cow<'a, str> {
        self.foot.clone()
    }

    pub fn head(&self) -> Cow<'a, str> {
        self.head.clone()
    }

    pub fn attach(&mut self, store: T) -> &mut Self {
        self.stores.push(store);
        self
    }

    pub fn set_name<S: Into<Cow<'a, str>>>(&mut self, name: S) -> &mut Self {
        self.name = name.into();
        self
    }

    pub fn set_hint<S: Into<Cow<'a, str>>>(&mut self, hint: S) -> &mut Self {
        self.hint = hint.into();
        self
    }

    pub fn set_help<S: Into<Cow<'a, str>>>(&mut self, help: S) -> &mut Self {
        self.help = help.into();
        self
    }

    pub fn set_foot<S: Into<Cow<'a, str>>>(&mut self, footer: S) -> &mut Self {
        self.foot = footer.into();
        self
    }

    pub fn set_head<S: Into<Cow<'a, str>>>(&mut self, header: S) -> &mut Self {
        self.head = header.into();
        self
    }
}

impl<T> Deref for Block<'_, T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.stores
    }
}

impl<T> DerefMut for Block<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stores
    }
}

impl<T> HelpDisplay for Block<'_, T> {
    fn gen_help<'a, P>(&self, policy: &P) -> Option<Cow<'a, str>>
    where
        Self: 'a,
        P: HelpPolicy<'a, Self>,
    {
        policy.format(self)
    }
}
