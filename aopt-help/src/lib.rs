pub mod block;
pub mod cmd;
pub mod error;
pub mod format;
pub mod store;
pub mod style;
pub mod wrapper;

pub use error::Error;
pub use error::Result;

pub mod prelude {
    pub use crate::block::Block;
    pub use crate::cmd::AddStore2Block;
    pub use crate::cmd::BlockMut;
    pub use crate::cmd::Command;
    pub use crate::format::DefaultAppPolicy;
    pub use crate::format::DefaultPolicy;
    pub use crate::format::HelpDisplay;
    pub use crate::format::HelpPolicy;
    pub use crate::store::Store;
    pub use crate::style::Align;
    pub use crate::style::Style;
    pub use crate::AppHelp;
}

use crate::block::Block;
use crate::cmd::Command;
use crate::format::DefaultAppPolicy;
use crate::format::DefaultPolicy;
use crate::format::HelpPolicy;
use crate::store::Store;
use crate::style::Style;

use std::io::Stdout;
use std::{borrow::Cow, io::Write};

#[derive(Debug, Clone)]
pub struct AppHelp<'a, W> {
    style: Style,

    writer: W,

    blocks: Vec<Block<'a, Cow<'a, str>>>, // store command sections

    cmds: Vec<Command<'a>>,

    global: usize,
}

impl<'a> Default for AppHelp<'a, Stdout> {
    fn default() -> Self {
        Self {
            style: Default::default(),
            writer: std::io::stdout(),
            blocks: Default::default(),
            cmds: Default::default(),
            global: 0,
        }
    }
}

impl<'a, W: Write> AppHelp<'a, W> {
    pub fn new<S: Into<Cow<'a, str>>>(name: S, head: S, foot: S, style: Style, writer: W) -> Self {
        Self {
            writer,
            style,
            blocks: vec![],
            cmds: vec![],
            global: 0,
        }
        .with_global(name, head, foot)
    }

    pub fn with_global<S: Into<Cow<'a, str>>>(mut self, name: S, head: S, foot: S) -> Self {
        let name = name.into();
        let hint = Cow::from("");
        let help = Cow::from("");
        let head = head.into();
        let foot = foot.into();
        let global = Command::new(name, hint, help, head, foot);

        self.cmds.push(global);
        self.global = self.cmds.len() - 1;
        self
    }

    pub fn foot(&self) -> Cow<'a, str> {
        self.global().foot()
    }

    pub fn head(&self) -> Cow<'a, str> {
        self.global().head()
    }

    pub fn name(&self) -> Cow<'a, str> {
        self.global().name()
    }

    pub fn style(&self) -> &Style {
        &self.style
    }

    pub fn global(&self) -> &Command<'a> {
        &self.cmds[self.global]
    }

    pub fn global_mut(&mut self) -> &mut Command<'a> {
        &mut self.cmds[self.global]
    }

    pub fn block(&self) -> &[Block<'a, Cow<'a, str>>] {
        &self.blocks
    }

    pub fn block_mut(&mut self) -> &mut [Block<'a, Cow<'a, str>>] {
        &mut self.blocks
    }

    pub fn has_cmd(&self) -> bool {
        !self.cmds.is_empty()
    }

    pub fn has_pos(&self) -> bool {
        self.cmds.iter().any(|cmd| cmd.has_position())
    }

    pub fn find_cmd<S: Into<Cow<'a, str>>>(&self, cmd: S) -> Option<&Command<'a>> {
        let name = cmd.into();

        self.cmds.iter().find(|v| v.name() == name)
    }

    pub fn find_cmd_mut<S: Into<Cow<'a, str>>>(&mut self, cmd: S) -> Option<&mut Command<'a>> {
        let name = cmd.into();

        self.cmds.iter_mut().find(|v| v.name() == name)
    }

    pub fn find_block<S: Into<Cow<'a, str>>>(&self, block: S) -> Option<&Block<'a, Cow<'a, str>>> {
        let name = block.into();

        self.blocks.iter().find(|v| v.name() == name)
    }

    pub fn find_block_mut<S: Into<Cow<'a, str>>>(
        &mut self,
        block: S,
    ) -> Option<&mut Block<'a, Cow<'a, str>>> {
        let name = block.into();

        self.blocks.iter_mut().find(|v| v.name() == name)
    }

    pub fn with_name<S: Into<Cow<'a, str>>>(mut self, name: S) -> Self {
        self.global_mut().set_name(name);
        self
    }

    pub fn with_head<S: Into<Cow<'a, str>>>(mut self, head: S) -> Self {
        self.global_mut().set_head(head);
        self
    }

    pub fn with_foot<S: Into<Cow<'a, str>>>(mut self, foot: S) -> Self {
        self.global_mut().set_foot(foot);
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_writer(mut self, writer: W) -> Self {
        self.writer = writer;
        self
    }

    pub fn set_name<S: Into<Cow<'a, str>>>(&mut self, name: S) -> &mut Self {
        self.global_mut().set_name(name);
        self
    }

    pub fn set_head<S: Into<Cow<'a, str>>>(&mut self, head: S) -> &mut Self {
        self.global_mut().set_head(head);
        self
    }

    pub fn set_foot<S: Into<Cow<'a, str>>>(&mut self, foot: S) -> &mut Self {
        self.global_mut().set_foot(foot);
        self
    }

    pub fn set_style(&mut self, style: Style) -> &mut Self {
        self.style = style;
        self
    }

    pub fn set_write(&mut self, writer: W) -> &mut Self {
        self.writer = writer;
        self
    }

    pub fn add_block(&mut self, block: Block<'a, Cow<'a, str>>) -> Result<&mut Self> {
        if self.find_block(block.name()).is_some() {
            Err(Error::DuplicatedBlockName(block.name().to_string()))
        } else {
            self.blocks.push(block);
            Ok(self)
        }
    }

    pub fn add_cmd<S: Into<Cow<'a, str>>>(
        &mut self,
        block: S,
        cmd: Command<'a>,
    ) -> Result<&mut Self> {
        let block = block.into();

        self.find_block_mut(block.clone())
            .ok_or_else(|| Error::InvalidBlockName(block.to_string()))?
            .push(cmd.name());
        if self.find_cmd(cmd.name()).is_some() {
            Err(Error::DuplicatedCommandName(cmd.name().to_string()))
        } else {
            self.cmds.push(cmd);
            Ok(self)
        }
    }

    pub fn new_block<S: Into<Cow<'a, str>>>(&mut self, name: S) -> Result<AddBlock2App<'a, '_>> {
        let name = name.into();

        if self.find_block(name.clone()).is_some() {
            Err(Error::DuplicatedBlockName(name.to_string()))
        } else {
            Ok(AddBlock2App::new(&mut self.blocks, name))
        }
    }

    pub fn new_cmd<S: Into<Cow<'a, str>>>(
        &mut self,
        block: S,
        name: S,
    ) -> Result<AddCmd2App<'a, '_, W>> {
        let name = name.into();
        let block = block.into();

        self.find_block_mut(block.clone())
            .ok_or_else(|| Error::InvalidBlockName(block.to_string()))?;
        if self.find_cmd(name.clone()).is_some() {
            Err(Error::DuplicatedCommandName(name.to_string()))
        } else {
            Ok(AddCmd2App::new(self, block, name))
        }
    }

    pub fn display(&mut self, show_global: bool) -> Result<()> {
        let policy = DefaultAppPolicy::new(vec![], show_global);
        let help = policy.format(self).ok_or_else(|| {
            Error::raise("Can not format app help with DefaultAppPolicy".to_string())
        })?;

        write!(&mut self.writer, "{}", help)
            .map_err(|e| Error::raise(format!("Can not write to handler: {:?}", e)))
    }

    pub fn display_with<P>(&mut self, policy: P) -> Result<()>
    where
        P: HelpPolicy<'a, Self>,
    {
        let help = policy
            .format(self)
            .ok_or_else(|| Error::raise("Can not format app help with given policy".to_string()))?;

        write!(&mut self.writer, "{}", help)
            .map_err(|e| Error::raise(format!("Can not write to handler: {:?}", e)))
    }

    pub fn display_cmd<S>(&mut self, cmd: S) -> Result<()>
    where
        S: Into<Cow<'a, str>>,
    {
        let name = cmd.into();
        let cmd = self.cmds.iter().find(|v| v.name() == name).ok_or_else(|| {
            Error::raise(format!("Can not format help of {name} with DefaultPolicy"))
        })?;
        let policy = DefaultPolicy::new(self.name(), self.style.clone(), vec![], true);
        let help = policy.format(cmd).ok_or_else(|| todo!())?;

        write!(&mut self.writer, "{}\n", help)
            .map_err(|e| Error::raise(format!("Can not write to handler: {:?}", e)))
    }

    pub fn display_cmd_with<S, P>(&mut self, cmd: S, policy: P) -> Result<()>
    where
        P: HelpPolicy<'a, Command<'a>>,
        S: Into<Cow<'a, str>>,
    {
        let name = cmd.into();
        let cmd = self.cmds.iter().find(|v| v.name() == name).ok_or_else(|| {
            Error::raise(format!("Can not format help of {name} with given policy"))
        })?;
        let help = policy.format(cmd).ok_or_else(|| todo!())?;

        write!(&mut self.writer, "{}", help)
            .map_err(|e| Error::raise(format!("Can not write to handler: {:?}", e)))
    }
}

pub struct AddStore2App<'a, 'b> {
    store: Store<'a>,

    block: &'b mut Block<'a, Store<'a>>,

    added: bool,
}

impl<'a, 'b> AddStore2App<'a, 'b> {
    pub fn new<S: Into<Cow<'a, str>>>(block: &'b mut Block<'a, Store<'a>>, name: S) -> Self {
        let mut store = Store::default();

        store.set_optional(true);
        store.set_name(name);
        Self {
            block,
            store,
            added: false,
        }
    }

    pub fn set_optional(mut self, optional: bool) -> Self {
        self.store.set_optional(optional);
        self
    }

    pub fn set_position(mut self, position: bool) -> Self {
        self.store.set_position(position);
        self
    }

    pub fn set_hint<S: Into<Cow<'a, str>>>(mut self, hint: S) -> Self {
        self.store.set_hint(hint);
        self
    }

    pub fn set_help<S: Into<Cow<'a, str>>>(mut self, help: S) -> Self {
        self.store.set_help(help);
        self
    }

    pub fn set_name<S: Into<Cow<'a, str>>>(mut self, name: S) -> Self {
        self.store.set_name(name);
        self
    }

    pub fn set_type<S: Into<Cow<'a, str>>>(mut self, type_name: S) -> Self {
        self.store.set_type(type_name);
        self
    }

    pub fn submit(mut self) {
        if !self.added {
            let store = std::mem::take(&mut self.store);

            self.block.attach(store);
            self.added = true;
        }
    }
}

impl<'a, 'b> Drop for AddStore2App<'a, 'b> {
    fn drop(&mut self) {
        if !self.added {
            let store = std::mem::take(&mut self.store);

            self.block.attach(store);
            self.added = true;
        }
    }
}

pub struct AddBlock2App<'a, 'b> {
    block: Block<'a, Cow<'a, str>>,

    blocks: &'b mut Vec<Block<'a, Cow<'a, str>>>,

    added: bool,
}

impl<'a, 'b> AddBlock2App<'a, 'b> {
    pub fn new<S: Into<Cow<'a, str>>>(
        blocks: &'b mut Vec<Block<'a, Cow<'a, str>>>,
        name: S,
    ) -> Self {
        let mut block = Block::default();

        block.set_name(name);
        Self {
            blocks,
            block,
            added: false,
        }
    }

    pub fn attach<S: Into<Cow<'a, str>>>(&mut self, store: S) -> &mut Self {
        self.block.attach(store.into());
        self
    }

    pub fn set_hint<S: Into<Cow<'a, str>>>(mut self, hint: S) -> Self {
        self.block.set_hint(hint);
        self
    }

    pub fn set_help<S: Into<Cow<'a, str>>>(mut self, help: S) -> Self {
        self.block.set_help(help);
        self
    }

    pub fn set_name<S: Into<Cow<'a, str>>>(mut self, name: S) -> Self {
        self.block.set_name(name);
        self
    }

    pub fn set_head<S: Into<Cow<'a, str>>>(mut self, head: S) -> Self {
        self.block.set_head(head);
        self
    }

    pub fn set_foot<S: Into<Cow<'a, str>>>(mut self, foot: S) -> Self {
        self.block.set_foot(foot);
        self
    }

    pub fn submit(mut self) {
        if !self.added {
            let block = std::mem::take(&mut self.block);

            self.blocks.push(block);
            self.added = true;
        }
    }
}

impl<'a, 'b> Drop for AddBlock2App<'a, 'b> {
    fn drop(&mut self) {
        if !self.added {
            let block = std::mem::take(&mut self.block);

            self.blocks.push(block);
            self.added = true;
        }
    }
}

pub struct AddCmd2App<'a, 'b, W: Write> {
    cmd: Command<'a>,

    app: &'b mut AppHelp<'a, W>,

    block: Cow<'a, str>,

    added: bool,
}

impl<'a, 'b, W: Write> AddCmd2App<'a, 'b, W> {
    pub fn new<S: Into<Cow<'a, str>>>(app: &'b mut AppHelp<'a, W>, block: S, name: S) -> Self {
        let mut cmd = Command::default();

        cmd.set_name(name);
        Self {
            app,
            cmd,
            block: block.into(),
            added: false,
        }
    }

    pub fn inner(&mut self) -> &mut Command<'a> {
        &mut self.cmd
    }

    pub fn set_name<S: Into<Cow<'a, str>>>(&mut self, name: S) -> &mut Self {
        self.cmd.set_name(name);
        self
    }

    pub fn set_hint<S: Into<Cow<'a, str>>>(&mut self, hint: S) -> &mut Self {
        self.cmd.set_hint(hint);
        self
    }

    pub fn set_help<S: Into<Cow<'a, str>>>(&mut self, help: S) -> &mut Self {
        self.cmd.set_help(help);
        self
    }

    pub fn set_foot<S: Into<Cow<'a, str>>>(&mut self, foot: S) -> &mut Self {
        self.cmd.set_foot(foot);
        self
    }

    pub fn set_head<S: Into<Cow<'a, str>>>(&mut self, head: S) -> &mut Self {
        self.cmd.set_head(head);
        self
    }

    pub fn submit(mut self) {
        if !self.added {
            let store = std::mem::take(&mut self.cmd);

            self.app.add_cmd(self.block.clone(), store).unwrap();
            self.added = true;
        }
    }
}

impl<'a, 'b, W: Write> Drop for AddCmd2App<'a, 'b, W> {
    fn drop(&mut self) {
        if !self.added {
            let store = std::mem::take(&mut self.cmd);

            self.app.add_cmd(self.block.clone(), store).unwrap();
            self.added = true;
        }
    }
}
