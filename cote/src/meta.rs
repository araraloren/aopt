use aopt::prelude::*;
use aopt::Error;

pub trait IntoConfig {
    type Ret: Config + ConfigValue;

    fn into_config<Parser>(self, parser: &Parser) -> Result<Self::Ret, Error>
    where
        Parser: OptParser,
        Parser::Output: Information;
}

/// Hold the option information from configuration files.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OptionMeta<T>
where
    T: Clone,
{
    pub id: String,

    pub option: String,

    pub hint: Option<String>,

    pub help: Option<String>,

    pub action: Option<Action>,

    pub alias: Option<Vec<String>>,

    pub value: Option<Vec<T>>,
}

impl<T> OptionMeta<T>
where
    T: Clone,
{
    pub fn new<S: Into<String>>(id: S, option: S) -> Self {
        Self {
            id: id.into(),
            option: option.into(),
            hint: None,
            help: None,
            action: None,
            alias: None,
            value: None,
        }
    }

    pub fn id(&self) -> &String {
        &self.id
    }

    pub fn option(&self) -> &String {
        &self.option
    }

    pub fn hint(&self) -> Option<&String> {
        self.hint.as_ref()
    }

    pub fn help(&self) -> Option<&String> {
        self.help.as_ref()
    }

    pub fn action(&self) -> Option<&Action> {
        self.action.as_ref()
    }

    pub fn alias(&self) -> Option<&Vec<String>> {
        self.alias.as_ref()
    }

    pub fn value(&self) -> Option<&Vec<T>> {
        self.value.as_ref()
    }

    pub fn take_option(&mut self) -> String {
        std::mem::take(&mut self.option)
    }

    pub fn take_hint(&mut self) -> Option<String> {
        self.hint.take()
    }

    pub fn take_help(&mut self) -> Option<String> {
        self.help.take()
    }

    pub fn take_action(&mut self) -> Option<Action> {
        self.action.take()
    }

    pub fn take_alias(&mut self) -> Option<Vec<String>> {
        self.alias.take()
    }

    pub fn take_value(&mut self) -> Option<Vec<T>> {
        self.value.take()
    }

    pub fn with_id<S: Into<String>>(mut self, id: S) -> Self {
        self.id = id.into();
        self
    }

    pub fn with_option<S: Into<String>>(mut self, option: S) -> Self {
        self.option = option.into();
        self
    }

    pub fn with_hint<S: Into<String>>(mut self, hint: Option<S>) -> Self {
        self.hint = hint.map(|v| v.into());
        self
    }

    pub fn with_help<S: Into<String>>(mut self, help: Option<S>) -> Self {
        self.help = help.map(|v| v.into());
        self
    }

    pub fn with_action(mut self, action: Option<Action>) -> Self {
        self.action = action;
        self
    }

    pub fn with_alias<S: Into<String>>(mut self, alias: Option<Vec<S>>) -> Self {
        self.alias = alias.map(|alias| alias.into_iter().map(|v| v.into()).collect());
        self
    }

    pub fn with_value(mut self, value: Option<Vec<T>>) -> Self {
        self.value = value;
        self
    }

    pub fn set_id<S: Into<String>>(&mut self, id: S) -> &mut Self {
        self.id = id.into();
        self
    }

    pub fn set_option<S: Into<String>>(&mut self, option: S) -> &mut Self {
        self.option = option.into();
        self
    }

    pub fn set_hint<S: Into<String>>(&mut self, hint: Option<S>) -> &mut Self {
        self.hint = hint.map(|v| v.into());
        self
    }

    pub fn set_help<S: Into<String>>(&mut self, help: Option<S>) -> &mut Self {
        self.help = help.map(|v| v.into());
        self
    }

    pub fn set_action(&mut self, action: Option<Action>) -> &mut Self {
        self.action = action;
        self
    }

    pub fn set_alias<S: Into<String>>(&mut self, alias: Option<Vec<S>>) -> &mut Self {
        self.alias = alias.map(|alias| alias.into_iter().map(|v| v.into()).collect());
        self
    }

    pub fn set_value(&mut self, value: Option<Vec<T>>) -> &mut Self {
        self.value = value;
        self
    }

    pub fn merge_value(&mut self, other: &mut Self) -> &mut Self {
        match self.value.as_mut() {
            Some(value) => {
                if let Some(other_value) = other.value.as_mut() {
                    value.append(other_value);
                }
            }
            None => {
                self.value = std::mem::take(&mut other.value);
            }
        }
        self
    }
}

impl<T: ErasedTy + Clone> IntoConfig for OptionMeta<T> {
    type Ret = OptConfig;

    fn into_config<Parser>(mut self, parser: &Parser) -> Result<Self::Ret, Error>
    where
        Parser: OptParser,
        Parser::Output: Information,
    {
        let mut cfg = Self::Ret::new(parser, self.take_option().into())?;

        if let Some(hint) = self.take_hint() {
            cfg.set_hint(hint);
        }
        if let Some(help) = self.take_help() {
            cfg.set_help(help);
        }
        if let Some(action) = self.take_action() {
            cfg.set_action(action);
        }
        if let Some(values) = self.take_value() {
            cfg.set_initializer(ValInitializer::new_values(values));
        }
        if let Some(aliases) = self.take_alias() {
            for alias in aliases {
                cfg.add_alias(alias);
            }
        }
        Ok(cfg)
    }
}
