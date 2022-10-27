use crate::ctx::Ctx;
use crate::opt::Alias;
use crate::opt::Help;
use crate::opt::Index;
use crate::opt::Name;
use crate::opt::Opt;
use crate::opt::OptHelp;
use crate::opt::OptIndex;
use crate::opt::OptStyle;
use crate::opt::Optional;
use crate::opt::Prefix;
use crate::opt::ValPolicy;
use crate::opt::ValType;
use crate::opt::ValValidator;
use crate::ser::Services;
use crate::Arc;
use crate::Error;
use crate::RawVal;
use crate::Str;
use crate::Uid;

#[derive(Debug)]
pub struct UOpt {
    uid: Uid,

    name: Str,

    r#type: Str,

    help: OptHelp,

    prefix: Option<Str>,

    setted: bool,

    optional: bool,

    valtype: ValType,

    policy: ValPolicy,

    styles: Vec<OptStyle>,

    deactivate_style: bool,

    index: Option<OptIndex>,

    validator: ValValidator,

    alias: Option<Vec<(Str, Str)>>,
}

impl Opt for UOpt {
    fn ty(&self) -> Str {
        self.r#type.clone()
    }

    fn reset(&mut self) {
        self.set_setted(false);
    }

    fn valid(&self) -> bool {
        self.opt() || self.setted()
    }

    fn policy(&self) -> ValPolicy {
        self.policy
    }

    fn val_ty(&self) -> ValType {
        self.valtype
    }

    fn uid(&self) -> Uid {
        self.uid
    }

    fn set_uid(&mut self, uid: Uid) {
        self.uid = uid;
    }

    fn setted(&self) -> bool {
        self.setted
    }

    fn set_setted(&mut self, setted: bool) {
        self.setted = setted;
    }
    

    fn is_deact(&self) -> bool {
        self.deactivate_style
    }

    fn mat_sty(&self, style: OptStyle) -> bool {
        self.styles.iter().any(|v| v == &style)
    }

    fn check(
        &mut self,
        value: Option<Arc<RawVal>>,
        disable: bool,
        index: (usize, usize),
    ) -> Result<bool, Error> {
        self.validator.check(value, disable, index)
    }

    fn val_act(&mut self, val: Option<RawVal>, ser: &mut Services, ctx: &Ctx) -> Result<(), Error> {
        todo!()
    }
}

impl Name for UOpt {
    fn name(&self) -> &Str {
        &self.name
    }

    fn set_name(&mut self, name: Str) {
        self.name = name;
    }

    fn mat_name(&self, name: &Str) -> bool {
        self.name() == name
    }
}

impl Prefix for UOpt {
    fn pre(&self) -> Option<&Str> {
        self.prefix.as_ref()
    }

    fn set_pre(&mut self, prefix: Option<Str>) {
        self.prefix = prefix;
    }

    fn mat_pre(&self, prefix: Option<&Str>) -> bool {
        self.pre() == prefix
    }
}

impl Optional for UOpt {
    fn opt(&self) -> bool {
        self.optional
    }

    fn set_opt(&mut self, optional: bool) {
        self.optional = optional;
    }

    fn mat_opt(&self, optional: bool) -> bool {
        self.opt() == optional
    }
}

impl Alias for UOpt {
    fn alias(&self) -> Option<&Vec<(Str, Str)>> {
        self.alias.as_ref()
    }

    fn add_alias(&mut self, prefix: Str, name: Str) {
        if let Some(alias) = &mut self.alias {
            alias.push((prefix, name));
        }
    }

    fn rem_alias(&mut self, prefix: &Str, name: &Str) {
        if let Some(alias) = &mut self.alias {
            if let Some((i, _)) = alias
                .iter()
                .enumerate()
                .find(|(_, v)| &v.0 == prefix && &v.1 == name)
            {
                alias.remove(i);
            }
        }
    }

    fn mat_alias(&self, prefix: &Str, name: &Str) -> bool {
        if let Some(alias) = &self.alias {
            alias.iter().any(|v| &v.0 == prefix && &v.1 == name)
        } else {
            false
        }
    }
}

impl Help for UOpt {
    fn hint(&self) -> &Str {
        self.help.hint()
    }

    fn help(&self) -> &Str {
        self.help.help()
    }

    fn set_hint(&mut self, hint: Str) {
        self.help.set_hint(hint);
    }

    fn set_help(&mut self, help: Str) {
        self.help.set_help(help);
    }
}

impl Index for UOpt {
    fn idx(&self) -> Option<&OptIndex> {
        self.index.as_ref()
    }

    fn set_idx(&mut self, index: Option<OptIndex>) {
        self.index = index;
    }

    fn mat_idx(&self, index: Option<(usize, usize)>) -> bool {
        if let Some((index, total)) = index {
            if let Some(realindex) = self.idx() {
                if let Some(realindex) = realindex.calc_index(index, total) {
                    return realindex == index;
                }
            }
        }
        false
    }
}
