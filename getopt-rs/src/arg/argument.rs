
use crate::str::Str;
use crate::err::Result;


#[derive(Debug, Clone, Hash, Default)]
pub struct Argument<'a, 'b> {
    pub current: Option<Str<'a>>,

    pub next: Option<Str<'a>>,
    
    prefix: Option<Str<'b>>,

    name: Option<Str<'a>>,

    value: Option<Str<'a>>,
}

impl<'a, 'b, 'c: 'a> Argument<'a, 'b> {
    pub fn new(current: Option<Str<'c>>, next: Option<Str<'c>>) -> Self {
        Self {
            current,
            next,
            .. Self::default()
        }
    }

    pub fn get_prefix(&self) -> Option<&Str<'b>> {
        self.prefix.as_ref()
    }

    pub fn get_name(&self) -> Option<&Str<'a>> {
        self.name.as_ref()
    }

    pub fn get_value(&self) -> Option<&Str<'a>> {
        self.value.as_ref()
    }

    #[cfg(not(feature="async"))]
    pub fn parse(&mut self, prefix: &Vec<Str<'b>>) -> Result<bool> {
        Ok(true)
    }

    #[cfg(feature="async")]
    pub async fn parse(&mut self, prefix: &Vec<Str<'b>>) -> Result<bool> {
        Ok(true)
    }
}