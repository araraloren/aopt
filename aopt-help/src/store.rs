use std::fmt::Debug;

#[derive(Debug, Default)]
pub struct Store {
    sections: Vec<SecStore>,

    cmds: Vec<CmdStore>,

    global_cmd: CmdStore,
}

impl Store {
    pub fn new_sec<S: Into<String>>(&mut self, sec: S) -> SectionMut {
        SectionMut::new(self, sec)
    }

    pub fn new_cmd(&mut self, cmd: &str) -> CmdMut {
        CmdMut::new(self, cmd)
    }

    pub fn new_pos(&mut self, cmd: &str, pos: &str) -> Option<CmdPosMut> {
        self.get_cmd_mut(cmd).map(|v| CmdPosMut::new(v, pos))
    }

    pub fn new_opt(&mut self, cmd: &str, opt: &str) -> Option<CmdOptMut> {
        self.get_cmd_mut(cmd).map(|v| CmdOptMut::new(v, opt))
    }

    pub fn attach_cmd(&mut self, sec: &str, cmd: &str) -> bool {
        if self.get_cmd(cmd).is_some() {
            if let Some(sec_store) = self.get_sec_mut(sec) {
                sec_store.attach_cmd(cmd);
                return true;
            }
        }
        false
    }

    pub fn add_sec(&mut self, sec: SecStore) {
        self.sections.push(sec)
    }

    pub fn add_cmd(&mut self, cmd: CmdStore) {
        self.cmds.push(cmd)
    }

    pub fn add_pos(&mut self, cmd: &str, pos: PosStore) -> bool {
        if let Some(cmd_store) = self.get_cmd_mut(cmd) {
            cmd_store.add_pos(pos);
            return true;
        }
        false
    }

    pub fn add_opt(&mut self, cmd: &str, opt: OptStore) -> bool {
        if let Some(cmd_store) = self.get_cmd_mut(cmd) {
            cmd_store.add_opt(opt);
            return true;
        }
        false
    }

    pub fn sec_len(&self) -> usize {
        self.sections.len()
    }

    pub fn cmd_len(&self) -> usize {
        self.cmds.len()
    }

    pub fn get_secs(&self) -> &[SecStore] {
        &self.sections
    }

    pub fn get_cmds(&self) -> &[CmdStore] {
        &self.cmds
    }

    pub fn get_sec(&self, sec: &str) -> Option<&SecStore> {
        self.sections.iter().find(|&v| v.get_name() == sec)
    }

    pub fn get_cmd(&self, cmd: &str) -> Option<&CmdStore> {
        self.cmds.iter().find(|&v| v.get_name() == cmd)
    }

    pub fn get_pos(&self, cmd: &str, pos: &str) -> Option<&PosStore> {
        self.get_cmd(cmd).and_then(|v| v.get_pos(pos))
    }

    pub fn get_opt(&self, cmd: &str, opt: &str) -> Option<&OptStore> {
        self.get_cmd(cmd).and_then(|v| v.get_opt(opt))
    }

    pub fn get_sec_mut(&mut self, sec: &str) -> Option<&mut SecStore> {
        self.sections.iter_mut().find(|v| v.get_name() == sec)
    }

    pub fn get_cmd_mut(&mut self, cmd: &str) -> Option<&mut CmdStore> {
        self.cmds.iter_mut().find(|v| v.get_name() == cmd)
    }

    pub fn get_pos_mut(&mut self, cmd: &str, pos: &str) -> Option<&mut PosStore> {
        self.get_cmd_mut(cmd).and_then(|v| v.get_pos_mut(pos))
    }

    pub fn get_opt_mut(&mut self, cmd: &str, opt: &str) -> Option<&mut OptStore> {
        self.get_cmd_mut(cmd).and_then(|v| v.get_opt_mut(opt))
    }

    pub fn sec_iter(&self) -> std::slice::Iter<'_, SecStore> {
        self.sections.iter()
    }

    pub fn cmd_iter(&self) -> std::slice::Iter<'_, CmdStore> {
        self.cmds.iter()
    }

    pub fn sec_iter_mut(&mut self) -> std::slice::IterMut<'_, SecStore> {
        self.sections.iter_mut()
    }

    pub fn cmd_iter_mut(&mut self) -> std::slice::IterMut<'_, CmdStore> {
        self.cmds.iter_mut()
    }

    pub fn get_global(&self) -> &CmdStore {
        &self.global_cmd
    }

    pub fn get_global_mut(&mut self) -> &mut CmdStore {
        &mut self.global_cmd
    }
}

#[derive(Debug)]
pub struct SectionMut<'a> {
    g: &'a mut Store,
    s: SecStore,
}

impl<'a> SectionMut<'a> {
    pub fn new<S: Into<String>>(g: &'a mut Store, s: S) -> Self {
        let mut ss = SecStore::default();
        ss.set_name(s);
        Self { g, s: ss }
    }

    pub fn set_name<S: Into<String>>(&mut self, name: S) -> &mut Self {
        self.s.set_name(name);
        self
    }

    pub fn set_help<S: Into<String>>(&mut self, help: S) -> &mut Self {
        self.s.set_help(help);
        self
    }

    pub fn attach_cmd(&mut self, cmd: &str) -> &mut Self {
        self.s.attach_cmd(cmd);
        self
    }

    pub fn commit(&mut self) {
        self.g.add_sec(self.s.clone());
    }
}

#[derive(Debug)]
pub struct CmdMut<'a> {
    g: &'a mut Store,
    c: CmdStore,
}

impl<'a> CmdMut<'a> {
    pub fn new<S: Into<String>>(g: &'a mut Store, c: S) -> Self {
        let mut cs = CmdStore::default();
        cs.set_name(c);
        Self { g, c: cs }
    }

    pub fn set_name<S: Into<String>>(&mut self, name: S) -> &mut Self {
        self.c.set_name(name);
        self
    }

    pub fn set_footer<S: Into<String>>(&mut self, help: S) -> &mut Self {
        self.c.set_footer(help);
        self
    }

    pub fn set_header<S: Into<String>>(&mut self, help: S) -> &mut Self {
        self.c.set_header(help);
        self
    }

    pub fn set_hint<S: Into<String>>(&mut self, hint: S) -> &mut Self {
        self.c.set_hint(hint);
        self
    }

    pub fn set_help<S: Into<String>>(&mut self, help: S) -> &mut Self {
        self.c.set_help(help);
        self
    }

    pub fn add_pos(&mut self, pos: PosStore) -> &mut Self {
        self.c.add_pos(pos);
        self
    }

    pub fn add_opt(&mut self, opt: OptStore) -> &mut Self {
        self.c.add_opt(opt);
        self
    }

    pub fn new_pos<S: Into<String>>(&mut self, pos: S) -> CmdPosMut {
        CmdPosMut::new(&mut self.c, pos)
    }

    pub fn new_opt<S: Into<String>>(&mut self, opt: S) -> CmdOptMut {
        CmdOptMut::new(&mut self.c, opt)
    }

    pub fn commit(&mut self) {
        self.g.add_cmd(self.c.clone());
    }
}

#[derive(Debug)]
pub struct CmdPosMut<'a> {
    c: &'a mut CmdStore,
    p: PosStore,
}

impl<'a> CmdPosMut<'a> {
    pub fn new<S: Into<String>>(c: &'a mut CmdStore, p: S) -> Self {
        let mut ps = PosStore::default();
        ps.set_name(p);
        Self { c, p: ps }
    }

    pub fn set_name<S: Into<String>>(&mut self, name: S) -> &mut Self {
        self.p.set_name(name);
        self
    }

    pub fn set_hint<S: Into<String>>(&mut self, hint: S) -> &mut Self {
        self.p.set_hint(hint);
        self
    }

    pub fn set_help<S: Into<String>>(&mut self, help: S) -> &mut Self {
        self.p.set_help(help);
        self
    }

    pub fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.p.set_optional(optional);
        self
    }

    pub fn set_index<S: Into<String>>(&mut self, index: S) -> &mut Self {
        self.p.set_index(index);
        self
    }

    pub fn commit(&mut self) {
        self.c.add_pos(self.p.clone());
    }
}

#[derive(Debug)]
pub struct CmdOptMut<'a> {
    c: &'a mut CmdStore,
    o: OptStore,
}

impl<'a> CmdOptMut<'a> {
    pub fn new<S: Into<String>>(c: &'a mut CmdStore, o: S) -> Self {
        let mut os = OptStore::default();
        os.set_name(o);
        Self { c, o: os }
    }

    pub fn set_name<S: Into<String>>(&mut self, name: S) -> &mut Self {
        self.o.set_name(name);
        self
    }

    pub fn set_hint<S: Into<String>>(&mut self, hint: S) -> &mut Self {
        self.o.set_hint(hint);
        self
    }

    pub fn set_help<S: Into<String>>(&mut self, help: S) -> &mut Self {
        self.o.set_help(help);
        self
    }

    pub fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.o.set_optional(optional);
        self
    }

    pub fn commit(&mut self) {
        self.c.add_opt(self.o.clone());
    }
}

#[derive(Debug, Default, Clone)]
pub struct OptStore {
    name: String,

    hint: String,

    help: String,

    type_name: String,

    optional: bool,
}

impl OptStore {
    pub fn new<S: Into<String>>(name: S, hint: S, help: S, type_name: S, optional: bool) -> Self {
        Self {
            name: name.into(),
            hint: hint.into(),
            help: help.into(),
            type_name: type_name.into(),
            optional,
        }
    }

    pub fn set_name<S: Into<String>>(&mut self, name: S) -> &mut Self {
        self.name = name.into();
        self
    }

    pub fn set_hint<S: Into<String>>(&mut self, hint: S) -> &mut Self {
        self.hint = hint.into();
        self
    }

    pub fn set_help<S: Into<String>>(&mut self, help: S) -> &mut Self {
        self.help = help.into();
        self
    }

    pub fn set_type_name<S: Into<String>>(&mut self, type_name: S) -> &mut Self {
        self.type_name = type_name.into();
        self
    }

    pub fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.optional = optional;
        self
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_hint(&self) -> &str {
        &self.hint
    }

    pub fn get_help(&self) -> &str {
        &self.help
    }

    pub fn get_type_name(&self) -> &str {
        &self.type_name
    }

    pub fn get_optional(&self) -> bool {
        self.optional
    }
}

#[derive(Debug, Default, Clone)]
pub struct PosStore {
    name: String,

    hint: String,

    help: String,

    index: String,

    optional: bool,
}

impl PosStore {
    pub fn new<S: Into<String>>(name: S, hint: S, help: S, index: S, optional: bool) -> Self {
        Self {
            name: name.into(),
            hint: hint.into(),
            help: help.into(),
            index: index.into(),
            optional,
        }
    }

    pub fn set_name<S: Into<String>>(&mut self, name: S) -> &mut Self {
        self.name = name.into();
        self
    }

    pub fn set_hint<S: Into<String>>(&mut self, hint: S) -> &mut Self {
        self.hint = hint.into();
        self
    }

    pub fn set_help<S: Into<String>>(&mut self, help: S) -> &mut Self {
        self.help = help.into();
        self
    }

    pub fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.optional = optional;
        self
    }

    pub fn set_index<S: Into<String>>(&mut self, index: S) -> &mut Self {
        self.index = index.into();
        self
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_hint(&self) -> &str {
        &self.hint
    }

    pub fn get_help(&self) -> &str {
        &self.help
    }

    pub fn get_optional(&self) -> bool {
        self.optional
    }

    pub fn get_index(&self) -> &str {
        &self.index
    }
}

#[derive(Debug, Default, Clone)]
pub struct CmdStore {
    name: String,

    footer: String,

    header: String,

    hint: String,

    help: String,

    pos_store: Vec<PosStore>,

    opt_store: Vec<OptStore>,
}

impl CmdStore {
    pub fn new<S: Into<String>>(name: S, footer: S, header: S, hint: S, help: S) -> Self {
        Self {
            name: name.into(),
            footer: footer.into(),
            header: header.into(),
            hint: hint.into(),
            help: help.into(),
            pos_store: vec![],
            opt_store: vec![],
        }
    }

    pub fn set_name<S: Into<String>>(&mut self, name: S) -> &mut Self {
        self.name = name.into();
        self
    }

    pub fn set_footer<S: Into<String>>(&mut self, help: S) -> &mut Self {
        self.footer = help.into();
        self
    }

    pub fn set_header<S: Into<String>>(&mut self, help: S) -> &mut Self {
        self.header = help.into();
        self
    }

    pub fn set_hint<S: Into<String>>(&mut self, hint: S) -> &mut Self {
        self.hint = hint.into();
        self
    }

    pub fn set_help<S: Into<String>>(&mut self, help: S) -> &mut Self {
        self.help = help.into();
        self
    }

    pub fn add_pos(&mut self, pos: PosStore) -> &mut Self {
        self.pos_store.push(pos);
        self
    }

    pub fn add_opt(&mut self, opt: OptStore) -> &mut Self {
        self.opt_store.push(opt);
        self
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_header(&self) -> &str {
        self.header.as_str()
    }

    pub fn get_footer(&self) -> &str {
        self.footer.as_str()
    }

    pub fn get_hint(&self) -> &str {
        self.hint.as_str()
    }

    pub fn get_help(&self) -> &str {
        self.help.as_str()
    }

    pub fn get_pos(&self, pos: &str) -> Option<&PosStore> {
        self.pos_store.iter().find(|&v| v.name == pos)
    }

    pub fn get_opt(&self, opt: &str) -> Option<&OptStore> {
        self.opt_store.iter().find(|&v| v.name == opt)
    }

    pub fn get_pos_mut(&mut self, pos: &str) -> Option<&mut PosStore> {
        self.pos_store.iter_mut().find(|v| v.name == pos)
    }

    pub fn get_opt_mut(&mut self, opt: &str) -> Option<&mut OptStore> {
        self.opt_store.iter_mut().find(|v| v.name == opt)
    }

    pub fn pos_len(&self) -> usize {
        self.pos_store.len()
    }

    pub fn opt_len(&self) -> usize {
        self.opt_store.len()
    }

    pub fn get_pos_store(&self) -> &[PosStore] {
        &self.pos_store
    }

    pub fn get_opt_store(&self) -> &[OptStore] {
        &self.opt_store
    }

    pub fn pos_iter(&self) -> std::slice::Iter<'_, PosStore> {
        self.pos_store.iter()
    }

    pub fn opt_iter(&self) -> std::slice::Iter<'_, OptStore> {
        self.opt_store.iter()
    }
}

#[derive(Debug, Default, Clone)]
pub struct SecStore {
    name: String,

    help: String,

    cmd_attach: Vec<String>,
}

impl SecStore {
    pub fn set_name<S: Into<String>>(&mut self, name: S) -> &mut Self {
        self.name = name.into();
        self
    }

    pub fn set_help<S: Into<String>>(&mut self, help: S) -> &mut Self {
        self.help = help.into();
        self
    }

    pub fn attach_cmd<S: Into<String>>(&mut self, cmd: S) -> &mut Self {
        self.cmd_attach.push(cmd.into());
        self
    }

    pub fn has_cmd(&self, cmd: &str) -> bool {
        self.cmd_attach.iter().any(|v| v == &cmd)
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_help(&self) -> &str {
        self.help.as_str()
    }

    pub fn cmd_len(&self) -> usize {
        self.cmd_attach.len()
    }

    pub fn get_cmd(&self) -> &[String] {
        &self.cmd_attach
    }

    pub fn cmd_iter(&self) -> std::slice::Iter<'_, String> {
        self.cmd_attach.iter()
    }
}
