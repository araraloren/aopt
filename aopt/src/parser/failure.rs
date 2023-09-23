use std::ops::Deref;
use std::ops::DerefMut;

use crate::err::ErrorCmd;
use crate::Error;

#[derive(Debug, Default)]
pub struct FailManager {
    fails: Vec<Error>,
}

impl FailManager {
    pub fn push(&mut self, err: Error) -> &mut Self {
        self.fails.push(err);
        self
    }

    pub fn cause(mut self, new_err: Error) -> Error {
        if self.is_empty() {
            new_err
        } else {
            let mut err = self.fails.remove(0);

            for fail in self.fails {
                err = err.cause(fail);
            }
            new_err.cause_by(err)
        }
    }

    pub fn cause_uid(self, new_err: Error) -> Error {
        if self.is_empty() {
            new_err
        } else {
            let mut fails = self
                .fails
                .into_iter()
                .filter(|v| new_err.uid() == v.uid() || v.uid().is_none());

            if let Some(fail) = fails.next() {
                let mut err = fail;

                for fail in fails {
                    err = err.cause(fail);
                }
                new_err.cause_by(err)
            } else {
                new_err
            }
        }
    }

    pub fn process_check<T, E>(self, ret: Result<T, E>) -> Result<T, Error>
    where
        E: Into<Error>,
    {
        match ret {
            Ok(v) => Ok(v),
            Err(e) => Err(self.cause_uid(e.into())),
        }
    }

    pub fn find_err_command(&self) -> Option<ErrorCmd> {
        for fail in self.fails.iter() {
            if let Some(error_cmd) = fail.command() {
                return Some(error_cmd);
            }
        }
        None
    }
}

impl Deref for FailManager {
    type Target = Vec<Error>;

    fn deref(&self) -> &Self::Target {
        &self.fails
    }
}

impl DerefMut for FailManager {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.fails
    }
}
