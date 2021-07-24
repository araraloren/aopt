
use std::marker::PhantomData;

use crate::prelude::*;
use crate::set::Commit;

#[derive(Debug)]
struct DataChecker {
    type_name: &'static str,

    deactivate_style: bool,

    cb_value: OptValue,

    default_value: OptValue,

    name: &'static str,

    prefix: &'static str,

    alias: Vec<(&'static str, &'static str)>,

    optional: bool,

    index: Option<OptIndex>,
}

impl Default for DataChecker {
    fn default() -> Self {
        Self {
            type_name: "",
            deactivate_style: false,
            cb_value: OptValue::default(),
            default_value: OptValue::default(),
            name: "",
            prefix: "",
            alias: vec![],
            optional: true,
            index: None,
        }
    }
}

impl DataChecker {
    pub fn check(&self, opt: &dyn Opt, cb_value: &OptValue) {
        assert_eq!(opt.get_name(), self.name);
        assert_eq!(opt.is_need_invoke(), true);
        assert_eq!(opt.get_optional(), self.optional);
        assert!(self.default_value.eq(opt.get_default_value()));
        assert_eq!(opt.get_type_name(), self.type_name);
        assert_eq!(self.deactivate_style, opt.is_deactivate_style());
        assert_eq!(self.prefix, opt.get_prefix());
        assert_eq!(opt.get_index(), self.index.as_ref());
        for (prefix, name) in &self.alias {
            assert!(opt.match_alias(prefix, name));
        }
        assert!(self.cb_value.eq(cb_value));
    }
}

pub struct TestingCase<S: Set, P: Parser<S>> {
    opt_str: &'static str,

    ret_value: Option<OptValue>,

    commit_tweak: Option<Box<dyn FnMut(&mut Commit)>>,

    callback_tweak: Option<Box<dyn FnMut(&mut P, Uid, Option<DataChecker>)>>,

    checker: Option<DataChecker>,

    marker: PhantomData<S>,
}

impl<S: Set, P: Parser<S>> TestingCase<S, P> {
    pub fn do_test(&mut self, set: &mut S, parser: &mut P) -> Result<()> {
        let mut commit = set.add_opt(self.opt_str)?;
        
        if let Some(tweak) = self.commit_tweak.as_mut() {
            tweak.as_mut()(&mut commit);
        }
        let uid = commit.commit()?;

        if let Some(tweak) = self.callback_tweak.as_mut() {
            tweak.as_mut()(parser, uid, self.checker.take());
        }
        Ok(())
    }

    pub fn check_ret(&mut self, set: &mut S) -> Result<()> {
        if let Some(ret_value) = self.ret_value.as_ref() {
            if let Some(opt) = set.filter(self.opt_str)?.find() {
                assert!(ret_value.eq(opt.as_ref().get_value()));
            }
        }
        Ok(())
    }
}