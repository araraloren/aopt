use crate::Error;

pub trait OptValidator {
    fn check_name(
        &mut self,
        name: &str,
    ) -> Result<bool, Error>;
}

impl<Func> OptValidator for Func
where
    Func: FnMut(&str) -> Result<bool, Error>,
{
    fn check_name(
        &mut self,
        name: &str,
    ) -> Result<bool, Error> {
        (self)(name)
    }
}

#[derive(Debug, Clone)]
pub struct PrefixOptValidator<'a>(Vec<&'a str>);

impl<'a> Default for PrefixOptValidator<'a> {
    fn default() -> Self {
        Self::new(vec!["--/", "--", "-/", "-", "/"])
    }
}

impl<'a> PrefixOptValidator<'a> {
    pub fn new(prefix: Vec<&'a str>) -> Self {
        // sort the prefix by length
        let mut _self = Self(prefix);

        _self.sort_prefix();
        _self
    }

    fn sort_prefix(&mut self) {
        self.0.sort_by_key(|b| std::cmp::Reverse(b.len()));
    }

    pub fn add_prefix(&mut self, prefix: &'a str) -> &mut Self {
        self.0.push(prefix);
        self.sort_prefix();
        self
    }
}

impl<'a> OptValidator for PrefixOptValidator<'a> {
    fn check_name(&mut self, name: &str) -> Result<bool, Error> {
        for prefix in self.0.iter() {
            if name.starts_with(*prefix) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}
