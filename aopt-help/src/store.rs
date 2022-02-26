use std::fmt::Debug;
use ustr::Ustr;

#[derive(Debug, Default)]
pub struct Store {
    sections: Vec<SecStore>,

    cmds: Vec<CmdStore>,

    global_cmd: CmdStore,
}

impl Store {
    pub fn new_sec(&mut self, sec: Ustr) -> SectionMut {
        SectionMut::new(self, sec)
    }

    pub fn new_cmd(&mut self, cmd: Ustr) -> CmdMut {
        CmdMut::new(self, cmd)
    }

    pub fn new_pos(&mut self, cmd: Ustr, pos: Ustr) -> Option<CmdPosMut> {
        self.get_cmd_mut(cmd).map(|v| CmdPosMut::new(v, pos))
    }

    pub fn new_opt(&mut self, cmd: Ustr, opt: Ustr) -> Option<CmdOptMut> {
        self.get_cmd_mut(cmd).map(|v| CmdOptMut::new(v, opt))
    }

    pub fn attach_cmd(&mut self, sec: Ustr, cmd: Ustr) -> bool {
        if let Some(_) = self.get_cmd(cmd) {
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

    pub fn add_pos(&mut self, cmd: Ustr, pos: PosStore) -> bool {
        if let Some(cmd_store) = self.get_cmd_mut(cmd) {
            cmd_store.add_pos(pos);
            return true;
        }
        false
    }

    pub fn add_opt(&mut self, cmd: Ustr, opt: OptStore) -> bool {
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

    pub fn get_sec(&self, sec: Ustr) -> Option<&SecStore> {
        self.sections.iter().find(|&v| v.get_name() == sec)
    }

    pub fn get_cmd(&self, cmd: Ustr) -> Option<&CmdStore> {
        self.cmds.iter().find(|&v| v.get_name() == &cmd)
    }

    pub fn get_pos(&self, cmd: Ustr, pos: Ustr) -> Option<&PosStore> {
        self.get_cmd(cmd).and_then(|v| v.get_pos(pos))
    }

    pub fn get_opt(&self, cmd: Ustr, opt: Ustr) -> Option<&OptStore> {
        self.get_cmd(cmd).and_then(|v| v.get_opt(opt))
    }

    pub fn get_sec_mut(&mut self, sec: Ustr) -> Option<&mut SecStore> {
        self.sections.iter_mut().find(|v| v.get_name() == sec)
    }

    pub fn get_cmd_mut(&mut self, cmd: Ustr) -> Option<&mut CmdStore> {
        self.cmds.iter_mut().find(|v| v.get_name() == &cmd)
    }

    pub fn get_pos_mut(&mut self, cmd: Ustr, pos: Ustr) -> Option<&mut PosStore> {
        self.get_cmd_mut(cmd).and_then(|v| v.get_pos_mut(pos))
    }

    pub fn get_opt_mut(&mut self, cmd: Ustr, opt: Ustr) -> Option<&mut OptStore> {
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
    pub fn new(g: &'a mut Store, s: Ustr) -> Self {
        let mut ss = SecStore::default();
        ss.set_name(s);
        Self { g, s: ss }
    }

    pub fn set_name(&mut self, name: Ustr) -> &mut Self {
        self.s.set_name(name);
        self
    }

    pub fn set_help(&mut self, help: Ustr) -> &mut Self {
        self.s.set_help(help);
        self
    }

    pub fn attach_cmd(&mut self, cmd: Ustr) -> &mut Self {
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
    pub fn new(g: &'a mut Store, c: Ustr) -> Self {
        let mut cs = CmdStore::default();
        cs.set_name(c);
        Self { g, c: cs }
    }

    pub fn set_name(&mut self, name: Ustr) -> &mut Self {
        self.c.set_name(name);
        self
    }

    pub fn set_footer(&mut self, help: Ustr) -> &mut Self {
        self.c.set_footer(help);
        self
    }

    pub fn set_header(&mut self, help: Ustr) -> &mut Self {
        self.c.set_header(help);
        self
    }

    pub fn set_hint(&mut self, hint: Ustr) -> &mut Self {
        self.c.set_hint(hint);
        self
    }

    pub fn set_help(&mut self, help: Ustr) -> &mut Self {
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

    pub fn new_pos(&mut self, pos: Ustr) -> CmdPosMut {
        CmdPosMut::new(&mut self.c, pos)
    }

    pub fn new_opt(&mut self, opt: Ustr) -> CmdOptMut {
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
    pub fn new(c: &'a mut CmdStore, p: Ustr) -> Self {
        let mut ps = PosStore::default();
        ps.set_name(p);
        Self { c, p: ps }
    }

    pub fn set_name(&mut self, name: Ustr) -> &mut Self {
        self.p.set_name(name);
        self
    }

    pub fn set_hint(&mut self, hint: Ustr) -> &mut Self {
        self.p.set_hint(hint);
        self
    }

    pub fn set_help(&mut self, help: Ustr) -> &mut Self {
        self.p.set_help(help);
        self
    }

    pub fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.p.set_optional(optional);
        self
    }

    pub fn set_index(&mut self, index: Ustr) -> &mut Self {
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
    pub fn new(c: &'a mut CmdStore, o: Ustr) -> Self {
        let mut os = OptStore::default();
        os.set_name(o);
        Self { c, o: os }
    }

    pub fn set_name(&mut self, name: Ustr) -> &mut Self {
        self.o.set_name(name);
        self
    }

    pub fn set_hint(&mut self, hint: Ustr) -> &mut Self {
        self.o.set_hint(hint);
        self
    }

    pub fn set_help(&mut self, help: Ustr) -> &mut Self {
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
    name: Ustr,

    hint: Ustr,

    help: Ustr,

    type_name: Ustr,

    optional: bool,
}

impl OptStore {
    pub fn new(name: Ustr, hint: Ustr, help: Ustr, type_name: Ustr, optional: bool) -> Self {
        Self {
            name,
            hint,
            help,
            type_name,
            optional,
        }
    }

    pub fn set_name(&mut self, name: Ustr) -> &mut Self {
        self.name = name;
        self
    }

    pub fn set_hint(&mut self, hint: Ustr) -> &mut Self {
        self.hint = hint;
        self
    }

    pub fn set_help(&mut self, help: Ustr) -> &mut Self {
        self.help = help;
        self
    }

    pub fn set_type_name(&mut self, type_name: Ustr) -> &mut Self {
        self.type_name = type_name;
        self
    }

    pub fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.optional = optional;
        self
    }

    pub fn get_name(&self) -> &Ustr {
        &self.name
    }

    pub fn get_hint(&self) -> &Ustr {
        &self.hint
    }

    pub fn get_help(&self) -> &Ustr {
        &self.help
    }

    pub fn get_type_name(&self) -> &Ustr {
        &self.type_name
    }

    pub fn get_optional(&self) -> bool {
        self.optional
    }
}

#[derive(Debug, Default, Clone)]
pub struct PosStore {
    name: Ustr,

    hint: Ustr,

    help: Ustr,

    index: Ustr,

    optional: bool,
}

impl PosStore {
    pub fn new(name: Ustr, hint: Ustr, help: Ustr, index: Ustr, optional: bool) -> Self {
        Self {
            name,
            hint,
            help,
            index,
            optional,
        }
    }

    pub fn set_name(&mut self, name: Ustr) -> &mut Self {
        self.name = name;
        self
    }

    pub fn set_hint(&mut self, hint: Ustr) -> &mut Self {
        self.hint = hint;
        self
    }

    pub fn set_help(&mut self, help: Ustr) -> &mut Self {
        self.help = help;
        self
    }

    pub fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.optional = optional;
        self
    }

    pub fn set_index(&mut self, index: Ustr) -> &mut Self {
        self.index = index;
        self
    }

    pub fn get_name(&self) -> &Ustr {
        &self.name
    }

    pub fn get_hint(&self) -> &Ustr {
        &self.hint
    }

    pub fn get_help(&self) -> &Ustr {
        &self.help
    }

    pub fn get_optional(&self) -> bool {
        self.optional
    }

    pub fn get_index(&self) -> &Ustr {
        &self.index
    }
}

#[derive(Debug, Default, Clone)]
pub struct CmdStore {
    name: Ustr,

    footer: Ustr,

    header: Ustr,

    hint: Ustr,

    help: Ustr,

    pos_store: Vec<PosStore>,

    opt_store: Vec<OptStore>,
}

impl CmdStore {
    pub fn new(name: Ustr, footer: Ustr, header: Ustr, hint: Ustr, help: Ustr) -> Self {
        Self {
            name,
            footer,
            header,
            hint,
            help,
            pos_store: vec![],
            opt_store: vec![],
        }
    }

    pub fn set_name(&mut self, name: Ustr) -> &mut Self {
        self.name = name;
        self
    }

    pub fn set_footer(&mut self, help: Ustr) -> &mut Self {
        self.footer = help;
        self
    }

    pub fn set_header(&mut self, help: Ustr) -> &mut Self {
        self.header = help;
        self
    }

    pub fn set_hint(&mut self, hint: Ustr) -> &mut Self {
        self.hint = hint;
        self
    }

    pub fn set_help(&mut self, help: Ustr) -> &mut Self {
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

    pub fn get_name(&self) -> &Ustr {
        &self.name
    }

    pub fn get_header(&self) -> &Ustr {
        &self.header
    }

    pub fn get_footer(&self) -> &Ustr {
        &self.footer
    }

    pub fn get_hint(&self) -> &Ustr {
        &self.hint
    }

    pub fn get_help(&self) -> &Ustr {
        &self.help
    }

    pub fn get_pos(&self, pos: Ustr) -> Option<&PosStore> {
        self.pos_store.iter().find(|&v| v.name == pos)
    }

    pub fn get_opt(&self, opt: Ustr) -> Option<&OptStore> {
        self.opt_store.iter().find(|&v| v.name == opt)
    }

    pub fn get_pos_mut(&mut self, pos: Ustr) -> Option<&mut PosStore> {
        self.pos_store.iter_mut().find(|v| v.name == pos)
    }

    pub fn get_opt_mut(&mut self, opt: Ustr) -> Option<&mut OptStore> {
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
    name: Ustr,

    help: Ustr,

    cmd_attach: Vec<Ustr>,
}

impl SecStore {
    pub fn set_name(&mut self, name: Ustr) -> &mut Self {
        self.name = name;
        self
    }

    pub fn set_help(&mut self, help: Ustr) -> &mut Self {
        self.help = help;
        self
    }

    pub fn attach_cmd(&mut self, cmd: Ustr) -> &mut Self {
        self.cmd_attach.push(cmd);
        self
    }

    pub fn has_cmd(&self, cmd: Ustr) -> bool {
        self.cmd_attach.iter().any(|v| v == &cmd)
    }

    pub fn get_name(&self) -> Ustr {
        self.name
    }

    pub fn get_help(&self) -> Ustr {
        self.help
    }

    pub fn cmd_len(&self) -> usize {
        self.cmd_attach.len()
    }

    pub fn get_cmd(&self) -> &[Ustr] {
        &self.cmd_attach
    }

    pub fn cmd_iter(&self) -> std::slice::Iter<'_, Ustr> {
        self.cmd_attach.iter()
    }
}
