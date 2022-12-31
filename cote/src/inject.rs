pub trait Inject<'a, P> {
    type Ret;
    type Error;

    fn inject(parser: &'a mut P) -> Result<Self::Ret, Self::Error>;
}

pub trait InjectFrom<'a, P> {
    type Ret;
    type Error;

    fn inject_from(&mut self, parser: &'a mut P) -> Result<Self::Ret, Self::Error>;
}

pub trait ExtractVal<'a, P>
where
    Self: 'a + Sized,
{
    type Error;

    fn extract_new(parser: &'a P) -> Result<Self, Self::Error>;
}

pub trait ExtractValMut<'a, P>
where
    Self: 'a + Sized,
{
    type Error;

    fn extract_new_mut(parser: &'a mut P) -> Result<Self, Self::Error>;
}

pub trait ExtractValFor<'a, P>
where
    Self: 'a + Sized,
{
    type Error;

    fn extract_for(&mut self, name: &str, parser: &'a P) -> Result<&mut Self, Self::Error>;
}

pub trait ExtractValForMut<'a, P>
where
    Self: 'a + Sized,
{
    type Error;

    fn extract_for_mut(&mut self, name: &str, parser: &'a mut P) -> Result<&mut Self, Self::Error>;
}
