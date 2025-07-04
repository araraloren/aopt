use std::borrow::Cow;

use crate::str::CowStrUtils;
use crate::{error, Error};

pub trait OptValidator {
    type Error: Into<Error>;

    /// Check the option string.
    fn check(&mut self, name: &str) -> Result<bool, Self::Error>;

    /// Split the option string into prefix and name.
    fn split<'a>(&self, name: &Cow<'a, str>) -> Result<(Cow<'a, str>, Cow<'a, str>), Self::Error>;
}

pub trait PrefixedValidator {
    type Error: Into<Error>;

    /// Register the prefix to current validator.
    fn reg_prefix(&mut self, val: &str) -> Result<(), Self::Error>;

    /// Unregister the prefix to current validator.
    fn unreg_prefix(&mut self, val: &str) -> Result<(), Self::Error>;
}

/// A prefixed validator used in [`Policy`](crate::parser::Policy) and [`InvokeGuess`](crate::guess::InvokeGuess).
///
/// The default prefixes are `--/`, `--`, `-/`, `-` and `/`(only for windows).
#[derive(Debug, Clone)]
pub struct PrefixOptValidator(Vec<String>);

#[cfg(target_os = "windows")]
impl Default for PrefixOptValidator {
    fn default() -> Self {
        Self::new(
            ["--/", "--", "-/", "-", "/"]
                .map(|v| v.to_string())
                .to_vec(),
        )
    }
}

#[cfg(not(target_os = "windows"))]
impl Default for PrefixOptValidator {
    fn default() -> Self {
        Self::new(["--/", "--", "-/", "-"].map(|v| v.to_string()).to_vec())
    }
}

impl PrefixOptValidator {
    pub fn new(prefix: Vec<String>) -> Self {
        // sort the prefix by length
        let mut _self = Self(prefix);

        _self.sort_prefix();
        _self
    }

    fn sort_prefix(&mut self) {
        self.0.sort_by_key(|b| std::cmp::Reverse(b.len()));
    }

    pub fn add_prefix(&mut self, prefix: &str) -> &mut Self {
        self.0.push(prefix.to_string());
        self.sort_prefix();
        self
    }
}

impl OptValidator for PrefixOptValidator {
    type Error = Error;

    fn check(&mut self, name: &str) -> Result<bool, Self::Error> {
        for prefix in self.0.iter() {
            if name.starts_with(prefix) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    fn split<'a>(&self, name: &Cow<'a, str>) -> Result<(Cow<'a, str>, Cow<'a, str>), Self::Error> {
        for prefix in self.0.iter() {
            if name.starts_with(prefix) {
                return Ok(name.split_at(prefix.len()));
            }
        }
        Err(error!(
            "can not split the {}: invalid option name string",
            name
        ))
    }
}

impl PrefixedValidator for PrefixOptValidator {
    type Error = Error;

    fn reg_prefix(&mut self, val: &str) -> Result<(), Self::Error> {
        if self.0.iter().any(|v| v == val) {
            Err(error!("the prefix already exist"))
        } else {
            self.add_prefix(val);
            Ok(())
        }
    }

    fn unreg_prefix(&mut self, val: &str) -> Result<(), Self::Error> {
        if let Some(index) = self.0.iter().position(|v| v == val) {
            self.0.remove(index);
            Ok(())
        } else {
            Err(error!("the prefix not exist"))
        }
    }
}
