use std::fmt::Debug;

#[derive(Debug, Default)]
pub struct Store {
    global: CmdStore,

    sections: Vec<SecStore>,

    cmds: Vec<CmdStore>,
}

impl Store {
    pub fn new_sec(&mut self, sec: String) -> SectionMut {
        SectionMut::new(self, sec)
    }

    pub fn new_cmd(&mut self, cmd: String) -> CmdMut {
        CmdMut::new(self, cmd)
    }

    pub fn new_pos(&mut self, cmd: &str, pos: String) -> Option<CmdPosMut> {
        self.get_cmd_mut(cmd).map(|v| CmdPosMut::new(v, pos))
    }

    pub fn new_opt(&mut self, cmd: &str, opt: String) -> Option<CmdOptMut> {
        self.get_cmd_mut(cmd).map(|v| CmdOptMut::new(v, opt))
    }

    pub fn attach_cmd(&mut self, sec: &str, cmd: String) -> bool {
        if let Some(_) = self.get_cmd(cmd.as_str()) {
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

    pub fn set_global(&mut self, cmd: CmdStore) {
        self.global = cmd;
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

    pub fn get_global(&self) -> &CmdStore {
        &self.global
    }

    pub fn get_global_mut(&mut self) -> &mut CmdStore {
        &mut self.global
    }
}

#[derive(Debug)]
pub struct SectionMut<'a> {
    g: &'a mut Store,
    s: SecStore,
}

impl<'a> SectionMut<'a> {
    pub fn new(g: &'a mut Store, s: String) -> Self {
        let mut ss = SecStore::default();
        ss.set_name(s);
        Self { g, s: ss }
    }

    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.s.set_name(name);
        self
    }

    pub fn set_hint(&mut self, hint: String) -> &mut Self {
        self.s.set_hint(hint);
        self
    }

    pub fn set_help(&mut self, help: String) -> &mut Self {
        self.s.set_help(help);
        self
    }

    pub fn attach_cmd(&mut self, cmd: String) -> &mut Self {
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
    pub fn new(g: &'a mut Store, c: String) -> Self {
        let mut cs = CmdStore::default();
        cs.set_name(c);
        Self { g, c: cs }
    }

    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.c.set_name(name);
        self
    }

    pub fn set_usage(&mut self, help: String) -> &mut Self {
        self.c.set_usage(help);
        self
    }

    pub fn set_footer(&mut self, help: String) -> &mut Self {
        self.c.set_footer(help);
        self
    }

    pub fn set_header(&mut self, help: String) -> &mut Self {
        self.c.set_header(help);
        self
    }

    pub fn set_hint(&mut self, hint: String) -> &mut Self {
        self.c.set_hint(hint);
        self
    }

    pub fn set_help(&mut self, help: String) -> &mut Self {
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

    pub fn new_pos(&mut self, pos: String) -> CmdPosMut {
        CmdPosMut::new(&mut self.c, pos)
    }

    pub fn new_opt(&mut self, opt: String) -> CmdOptMut {
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
    pub fn new(c: &'a mut CmdStore, p: String) -> Self {
        let mut ps = PosStore::default();
        ps.set_name(p);
        Self { c, p: ps }
    }

    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.p.set_name(name);
        self
    }

    pub fn set_hint(&mut self, hint: String) -> &mut Self {
        self.p.set_hint(hint);
        self
    }

    pub fn set_help(&mut self, help: String) -> &mut Self {
        self.p.set_help(help);
        self
    }

    pub fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.p.set_optional(optional);
        self
    }

    pub fn set_index(&mut self, index: i64) -> &mut Self {
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
    pub fn new(c: &'a mut CmdStore, o: String) -> Self {
        let mut os = OptStore::default();
        os.set_name(o);
        Self { c, o: os }
    }

    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.o.set_name(name);
        self
    }

    pub fn set_hint(&mut self, hint: String) -> &mut Self {
        self.o.set_hint(hint);
        self
    }

    pub fn set_help(&mut self, help: String) -> &mut Self {
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

    optional: bool,
}

impl OptStore {
    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.name = name;
        self
    }

    pub fn set_hint(&mut self, hint: String) -> &mut Self {
        self.hint = hint;
        self
    }

    pub fn set_help(&mut self, help: String) -> &mut Self {
        self.help = help;
        self
    }

    pub fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.optional = optional;
        self
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_hint(&self) -> &str {
        self.hint.as_str()
    }

    pub fn get_help(&self) -> &str {
        self.help.as_str()
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

    index: i64,

    optional: bool,
}

impl PosStore {
    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.name = name;
        self
    }

    pub fn set_hint(&mut self, hint: String) -> &mut Self {
        self.hint = hint;
        self
    }

    pub fn set_help(&mut self, help: String) -> &mut Self {
        self.help = help;
        self
    }

    pub fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.optional = optional;
        self
    }

    pub fn set_index(&mut self, index: i64) -> &mut Self {
        self.index = index;
        self
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_hint(&self) -> &str {
        self.hint.as_str()
    }

    pub fn get_help(&self) -> &str {
        self.help.as_str()
    }

    pub fn get_optional(&self) -> bool {
        self.optional
    }

    pub fn get_index(&self) -> i64 {
        self.index
    }
}

#[derive(Debug, Default, Clone)]
pub struct CmdStore {
    name: String,

    usage: String,

    footer: String,

    header: String,

    hint: String,

    help: String,

    pos_store: Vec<PosStore>,

    opt_store: Vec<OptStore>,
}

impl CmdStore {
    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.name = name;
        self
    }

    pub fn set_usage(&mut self, help: String) -> &mut Self {
        self.usage = help;
        self
    }

    pub fn set_footer(&mut self, help: String) -> &mut Self {
        self.footer = help;
        self
    }

    pub fn set_header(&mut self, help: String) -> &mut Self {
        self.header = help;
        self
    }

    pub fn set_hint(&mut self, hint: String) -> &mut Self {
        self.hint = hint;
        self
    }

    pub fn set_help(&mut self, help: String) -> &mut Self {
        self.help = help;
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
}

#[derive(Debug, Default, Clone)]
pub struct SecStore {
    name: String,

    hint: String,

    help: String,

    cmd_attach: Vec<String>,
}

impl SecStore {
    pub fn set_name(&mut self, name: String) -> &mut Self {
        self.name = name;
        self
    }

    pub fn set_hint(&mut self, hint: String) -> &mut Self {
        self.hint = hint;
        self
    }

    pub fn set_help(&mut self, help: String) -> &mut Self {
        self.help = help;
        self
    }

    pub fn attach_cmd(&mut self, cmd: String) -> &mut Self {
        self.cmd_attach.push(cmd);
        self
    }

    pub fn has_cmd(&self, cmd: &str) -> bool {
        self.cmd_attach.iter().find(|&v| v == cmd).is_some()
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_hint(&self) -> &str {
        self.hint.as_str()
    }

    pub fn get_help(&self) -> &str {
        self.help.as_str()
    }
}