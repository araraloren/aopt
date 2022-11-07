pub trait Serialize
where
    Self: serde::Serialize,
{
    type Output;

    type Error: Into<crate::Error>;

    fn serialize(&self) -> Result<Self::Output, Self::Error>;
}

pub trait Deserialize<'a>
where
    Self: serde::Deserialize<'a>,
{
    type Error: Into<crate::Error>;

    fn serialize(str: &'a str) -> Result<Self, Self::Error>;
}
