
use crate::opt::{ Opt, index::Index };

#[derive(Debug, Clone, Default)]
pub struct Filter {
    deactivate_style: Option<bool>,

    optional: Option<bool>,

    type_name: Option<String>,

    name: Option<String>,

    prefix: Option<String>,

    index: Option<Index>,
}

impl Filter {
    pub fn set_deactivate_style(&mut self, deactivate_style: bool) -> &mut Self {
        self.deactivate_style = Some(deactivate_style);
        self
    }

    pub fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.optional = Some(optional);
        self
    }

    pub fn set_type_name(&mut self, type_name: String) -> &mut Self {
        self.type_name = Some(type_name);
        self
    }

    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.name = Some(name);
        self
    }

    pub fn set_prefix(&mut self, prefix: String) -> &mut Self {
        self.prefix = Some(prefix);
        self
    }

    pub fn set_index(&mut self, index: Index) -> &mut Self {
        self.index = Some(index);
        self
    }

    pub fn get_deactivate_style(&self) -> bool {
        self.deactivate_style.unwrap()
    }

    pub fn get_optional(&self) -> bool {
        self.optional.unwrap()
    }

    pub fn get_type_name(&self) -> &String {
        self.type_name.as_ref().unwrap()
    }

    pub fn get_name(&self) -> &String {
        self.name.as_ref().unwrap()
    }

    pub fn get_prefix(&self) -> &String {
        self.prefix.as_ref().unwrap()
    }

    pub fn get_index(&self) -> &Index {
        self.index.as_ref().unwrap()
    }

    pub fn has_deactivate_style(&self) -> bool {
        self.deactivate_style.is_some()
    }

    pub fn has_optional(&self) -> bool {
        self.optional.is_some()
    }

    pub fn has_type_name(&self) -> bool {
        self.type_name.is_some()
    }

    pub fn has_name(&self) -> bool {
        self.name.is_some()
    }

    pub fn has_prefix(&self) -> bool {
        self.prefix.is_some()
    }

    pub fn has_index(&self) -> bool {
        self.index.is_some()
    }

    pub fn match_opt(&self, opt: &dyn Opt) -> bool {
        let mut ret = true;

        if ret && self.has_type_name() {
            ret = ret && (self.get_type_name() == opt.get_type_name());
        }
        if ret && self.has_prefix() {
            let mut matched = opt.match_prefix(self.get_prefix());

            if ! matched {
                if let Some(alias) = opt.get_alias().as_ref() {
                    for item in alias.iter() {
                        if item.0 == self.get_prefix() {
                            matched = true;
                            break;
                        }
                    }
                }
            }
            ret = ret && matched;
        }
        if ret && self.has_name() {
            let mut matched = opt.match_name(self.get_name());

            if ! matched {
                if let Some(alias) = opt.get_alias().as_ref() {
                    for item in alias.iter() {
                        if item.1 == self.get_name() {
                            matched = true;
                            break;
                        }
                    }
                }
            }
            ret = ret && matched;
        }
        if ret && self.has_index() {
            ret = ret && (self.get_index() == opt.get_index());
        }
        ret
    }
}