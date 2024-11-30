use std::borrow::Cow;

use crate::format::{HelpDisplay, HelpPolicy};

#[derive(Debug, Default, Clone)]
pub struct Store<'a> {
    name: Cow<'a, str>,

    hint: Cow<'a, str>,

    help: Cow<'a, str>,

    r#type: Cow<'a, str>,

    optional: bool,

    position: bool,
}

impl<'a> Store<'a> {
    pub fn new<S: Into<Cow<'a, str>>>(
        name: S,
        hint: S,
        help: S,
        r#type: S,
        optional: bool,
        position: bool,
    ) -> Self {
        Self {
            name: name.into(),
            hint: hint.into(),
            help: help.into(),
            r#type: r#type.into(),
            optional,
            position,
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

    pub fn optional(&self) -> bool {
        self.optional
    }

    pub fn position(&self) -> bool {
        self.position
    }

    pub fn r#type(&self) -> Cow<'a, str> {
        self.r#type.clone()
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

    pub fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.optional = optional;
        self
    }

    pub fn set_position(&mut self, position: bool) -> &mut Self {
        self.position = position;
        self
    }

    pub fn set_type<S: Into<Cow<'a, str>>>(&mut self, type_name: S) -> &mut Self {
        self.r#type = type_name.into();
        self
    }
}

impl HelpDisplay for Store<'_> {
    fn gen_help<'a, P>(&self, policy: &P) -> Option<Cow<'a, str>>
    where
        Self: 'a,
        P: HelpPolicy<'a, Self>,
    {
        policy.format(self)
    }
}
