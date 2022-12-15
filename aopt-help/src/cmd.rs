use std::borrow::Cow;
use std::ops::Deref;
use std::ops::DerefMut;

use crate::block::Block;
use crate::error::Error;
use crate::format::HelpDisplay;
use crate::format::HelpPolicy;
use crate::store::Store;

#[derive(Debug, Default, Clone)]
pub struct Command<'a> {
    name: Cow<'a, str>,

    hint: Cow<'a, str>,

    help: Cow<'a, str>,

    foot: Cow<'a, str>,

    head: Cow<'a, str>,

    blocks: Vec<Block<'a, Cow<'a, str>>>,

    stores: Vec<Store<'a>>,
}

impl<'a> Command<'a> {
    pub fn new<S: Into<Cow<'a, str>>>(name: S, hint: S, help: S, head: S, foot: S) -> Self {
        Self {
            name: name.into(),
            foot: foot.into(),
            head: head.into(),
            hint: hint.into(),
            help: help.into(),
            blocks: vec![],
            stores: vec![],
        }
    }

    pub fn name(&self) -> Cow<'a, str> {
        self.name.clone()
    }

    pub fn hint(&self) -> Cow<'a, str> {
        self.hint.clone()
    }

    pub fn help(&self) -> Cow<'a, str> {
        self.help.clone()
    }

    pub fn head(&self) -> Cow<'a, str> {
        self.head.clone()
    }

    pub fn foot(&self) -> Cow<'a, str> {
        self.foot.clone()
    }

    pub fn block(&self) -> &Vec<Block<'a, Cow<'a, str>>> {
        &self.blocks
    }

    pub fn block_mut(&mut self) -> &mut Vec<Block<'a, Cow<'a, str>>> {
        &mut self.blocks
    }

    pub fn has_position(&self) -> bool {
        self.stores.iter().any(|store| store.position())
    }

    pub fn find_store<S: Into<Cow<'a, str>>>(&self, name: S) -> Option<&Store<'a>> {
        let name = name.into();
        self.stores.iter().find(|v| v.name() == name)
    }

    pub fn find_store_mut<S: Into<Cow<'a, str>>>(&mut self, name: S) -> Option<&mut Store<'a>> {
        let name = name.into();
        self.stores.iter_mut().find(|v| v.name() == name)
    }

    pub fn find_block<S: Into<Cow<'a, str>>>(&self, name: S) -> Option<&Block<'a, Cow<'a, str>>> {
        let name = name.into();
        self.blocks.iter().find(|v| v.name() == name)
    }

    pub fn find_block_mut<S: Into<Cow<'a, str>>>(
        &mut self,
        name: S,
    ) -> Option<&mut Block<'a, Cow<'a, str>>> {
        let name = name.into();
        self.blocks.iter_mut().find(|v| v.name() == name)
    }

    pub fn set_name<S: Into<Cow<'a, str>>>(&mut self, name: S) -> &mut Self {
        self.name = name.into();
        self
    }

    pub fn set_hint<S: Into<Cow<'a, str>>>(&mut self, hint: S) -> &mut Self {
        self.hint = hint.into();
        self
    }

    pub fn set_help<S: Into<Cow<'a, str>>>(&mut self, help: S) -> &mut Self {
        self.help = help.into();
        self
    }

    pub fn set_foot<S: Into<Cow<'a, str>>>(&mut self, help: S) -> &mut Self {
        self.foot = help.into();
        self
    }

    pub fn set_head<S: Into<Cow<'a, str>>>(&mut self, help: S) -> &mut Self {
        self.head = help.into();
        self
    }

    pub fn add_store<S: Into<Cow<'a, str>>>(
        &mut self,
        block: S,
        store: Store<'a>,
    ) -> Result<&mut Self, Error> {
        let name: Cow<'a, str> = block.into();

        if self.find_store(name.clone()).is_some() {
            return Err(Error::DuplicatedStoreName(store.name().to_string()));
        } else {
            let block = self
                .find_block_mut(name.clone())
                .ok_or_else(|| Error::InvalidBlockName(name.to_string()))?;

            block.attach(store.name());
            self.stores.push(store);
            Ok(self)
        }
    }

    pub fn add_block(&mut self, block: Block<'a, Cow<'a, str>>) -> Result<&mut Self, Error> {
        let name = block.name();

        if self.find_block(name.clone()).is_some() {
            return Err(Error::DuplicatedBlockName(name.to_string()));
        } else {
            self.blocks.push(block);
            Ok(self)
        }
    }

    pub fn new_store<S: Into<Cow<'a, str>>>(
        &mut self,
        block: S,
        name: S,
    ) -> Result<AddStore2Block<'a, '_>, Error> {
        let name = name.into();

        if self.find_store(name.clone()).is_some() {
            return Err(Error::DuplicatedStoreName(name.to_string()));
        } else {
            let block = block.into();
            let block = self
                .blocks
                .iter_mut()
                .find(|v| v.name() == block)
                .ok_or_else(|| Error::InvalidBlockName(block.to_string()))?;
            let stores = &mut self.stores;

            Ok(AddStore2Block::new(block, stores, name))
        }
    }

    pub fn new_block<S: Into<Cow<'a, str>>>(&mut self, name: S) -> Result<BlockMut<'a, '_>, Error> {
        let name = name.into();

        if self.find_block(name.clone()).is_some() {
            return Err(Error::DuplicatedBlockName(name.to_string()));
        } else {
            Ok(BlockMut::new(self, name))
        }
    }
}

impl<'a> Deref for Command<'a> {
    type Target = Vec<Store<'a>>;

    fn deref(&self) -> &Self::Target {
        &self.stores
    }
}

impl<'a> DerefMut for Command<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stores
    }
}

impl<'b> HelpDisplay for Command<'b> {
    fn gen_help<'a, P>(&self, policy: &P) -> Option<Cow<'a, str>>
    where
        Self: 'a,
        P: HelpPolicy<'a, Self>,
    {
        policy.format(self)
    }
}

pub struct AddStore2Block<'a, 'b> {
    store: Store<'a>,

    block: &'b mut Block<'a, Cow<'a, str>>,

    stores: &'b mut Vec<Store<'a>>,

    added: bool,
}

impl<'a, 'b> AddStore2Block<'a, 'b> {
    pub fn new<S: Into<Cow<'a, str>>>(
        block: &'b mut Block<'a, Cow<'a, str>>,
        stores: &'b mut Vec<Store<'a>>,
        name: S,
    ) -> Self {
        let mut store = Store::default();

        store.set_name(name);
        Self {
            block,
            store,
            stores,
            added: false,
        }
    }

    pub fn set_optional(&mut self, optional: bool) -> &mut Self {
        self.store.set_optional(optional);
        self
    }

    pub fn set_position(&mut self, position: bool) -> &mut Self {
        self.store.set_position(position);
        self
    }

    pub fn set_hint<S: Into<Cow<'a, str>>>(&mut self, hint: S) -> &mut Self {
        self.store.set_hint(hint);
        self
    }

    pub fn set_help<S: Into<Cow<'a, str>>>(&mut self, help: S) -> &mut Self {
        self.store.set_help(help);
        self
    }

    pub fn set_name<S: Into<Cow<'a, str>>>(&mut self, name: S) -> &mut Self {
        self.store.set_name(name);
        self
    }

    pub fn set_type<S: Into<Cow<'a, str>>>(&mut self, type_name: S) -> &mut Self {
        self.store.set_type(type_name);
        self
    }

    pub fn submit(mut self) {
        if !self.added {
            let store = std::mem::take(&mut self.store);

            self.block.attach(store.name());
            self.stores.push(store);
            self.added = true;
        }
    }
}

impl<'a, 'b> Drop for AddStore2Block<'a, 'b> {
    fn drop(&mut self) {
        if !self.added {
            let store = std::mem::take(&mut self.store);

            self.block.attach(store.name());
            self.stores.push(store);
            self.added = true;
        }
    }
}

pub struct BlockMut<'a, 'b> {
    block: Block<'a, Cow<'a, str>>,

    cmd: &'b mut Command<'a>,

    added: bool,
}

impl<'a, 'b> BlockMut<'a, 'b> {
    pub fn new<S: Into<Cow<'a, str>>>(cmd: &'b mut Command<'a>, name: S) -> Self {
        let mut opt = Block::default();

        opt.set_name(name);
        Self {
            cmd,
            block: opt,
            added: false,
        }
    }

    pub fn set_hint<S: Into<Cow<'a, str>>>(&mut self, hint: S) -> &mut Self {
        self.block.set_hint(hint);
        self
    }

    pub fn set_help<S: Into<Cow<'a, str>>>(&mut self, help: S) -> &mut Self {
        self.block.set_help(help);
        self
    }

    pub fn set_foot<S: Into<Cow<'a, str>>>(&mut self, footer: S) -> &mut Self {
        self.block.set_foot(footer);
        self
    }

    pub fn set_head<S: Into<Cow<'a, str>>>(&mut self, header: S) -> &mut Self {
        self.block.set_head(header);
        self
    }

    pub fn set_name<S: Into<Cow<'a, str>>>(&mut self, name: S) -> &mut Self {
        self.block.set_name(name);
        self
    }

    pub fn new_store<S: Into<Cow<'a, str>>>(&mut self, store: S) -> AddStore2Block<'a, '_> {
        let block = &mut self.block;
        let stores = &mut self.cmd.stores;

        AddStore2Block::new(block, stores, store)
    }

    pub fn submit(mut self) {
        if !self.added {
            let block = std::mem::take(&mut self.block);

            self.cmd.add_block(block).unwrap();
            self.added = true;
        }
    }
}

impl<'a, 'b> Drop for BlockMut<'a, 'b> {
    fn drop(&mut self) {
        if !self.added {
            let block = std::mem::take(&mut self.block);
            let cmd_name = self.cmd.name();

            self.cmd
                .add_block(block)
                .expect(&format!("Can not add block to Command {cmd_name}"));
        }
    }
}
