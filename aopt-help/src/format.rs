mod policy;

use std::borrow::Cow;

pub use self::policy::DefaultAppPolicy;
pub use self::policy::DefaultPolicy;

pub trait HelpPolicy<'a, I> {
    fn format(&self, item: &I) -> Option<Cow<'a, str>>;
}

pub trait HelpDisplay
where
    Self: Sized,
{
    fn gen_help<'a, P>(&self, policy: &P) -> Option<Cow<'a, str>>
    where
        Self: 'a,
        P: HelpPolicy<'a, Self>;
}
