use super::Service;
use crate::err::Result;
use crate::opt::OptCallback;
use crate::set::Commit;
use crate::set::CreateInfo;
use crate::set::Set;
use crate::uid::Uid;
use std::cell::RefCell;
use std::ops::Deref;
use std::ops::DerefMut;

#[derive(Debug)]
pub struct CallbackCommit<'a, 'b, S: Set, SS: Service> {
    set_commit: Commit<'a, S>,
    service_ref: &'b mut SS,
    callback: OptCallback,
}

impl<'a, 'b, S: Set, SS: Service> Deref for CallbackCommit<'a, 'b, S, SS> {
    type Target = Commit<'a, S>;

    fn deref(&self) -> &Self::Target {
        &self.set_commit
    }
}

impl<'a, 'b, S: Set, SS: Service> DerefMut for CallbackCommit<'a, 'b, S, SS> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.set_commit
    }
}

impl<'a, 'b, S: Set, SS: Service> CallbackCommit<'a, 'b, S, SS> {
    pub fn new(
        set: &'a mut S,
        service: &'b mut SS,
        info: CreateInfo,
        callback: OptCallback,
    ) -> Self {
        Self {
            set_commit: Commit::new(set, info),
            service_ref: service,
            callback,
        }
    }

    pub fn commit(&mut self) -> Result<Uid> {
        let uid = self.set_commit.commit()?;
        self.service_ref
            .get_callback_mut()
            .insert(uid, RefCell::new(std::mem::take(&mut self.callback)));
        Ok(uid)
    }
}
