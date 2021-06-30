use std::fmt::Debug;

pub type Uid = u64;

pub trait Generator: Debug {
    /// Get current uid .
    fn get(&self) -> Uid;

    fn set(&mut self, uid: u64);

    fn acq(&mut self) -> Uid;

    fn rel(&mut self);

    fn skip(&mut self, offset: u64) {
        self.set(self.get() + offset);
    }

    fn reset(&mut self) {
        self.set(0);
    }
}

#[derive(Debug, Default)]
pub struct UidGenerator {
    uid: Uid,
}

impl UidGenerator {
    pub fn new(uid: Uid) -> Self {
        Self { uid }
    }
}

impl Generator for UidGenerator {
    fn get(&self) -> Uid {
        self.uid
    }

    fn set(&mut self, uid: u64) {
        self.uid = uid;
    }

    fn acq(&mut self) -> Uid {
        self.uid += 1;
        self.uid
    }

    fn rel(&mut self) {
        self.uid -= 1;
    }
}

impl From<u64> for UidGenerator {
    fn from(v: u64) -> Self {
        Self { uid: v.into() }
    }
}

#[cfg(test)]
mod test {
    use crate::uid::Generator;
    use crate::uid::UidGenerator;

    #[test]
    fn make_sure_uid_work() {
        let mut gen = UidGenerator::new(0);

        assert_eq!(gen.acq(), 1);
        assert_eq!(gen.acq(), 2);
        assert_eq!(gen.acq(), 3);

        gen.rel();

        assert_eq!(gen.get(), 2);
        assert_eq!(gen.acq(), 3);

        gen.set(6);

        assert_eq!(gen.acq(), 7);
        assert_eq!(gen.acq(), 8);

        gen.skip(8);

        assert_eq!(gen.acq(), 17);
        assert_eq!(gen.acq(), 18);

        gen.reset();

        assert_eq!(gen.acq(), 1);
        assert_eq!(gen.acq(), 2);
    }
}
