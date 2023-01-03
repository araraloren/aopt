use syn::{parse::Parse, Attribute};
pub(crate) use crate::CfgValue;

#[derive(Debug, Clone)]
pub enum CfgKind {
    Policy,
}

#[derive(Debug, Clone)]
pub(crate) struct GlobalCfg {
    pub kind: CfgKind,

    pub value: CfgValue,
}

impl Parse for GlobalCfg {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        todo!()
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct GlobalCfgs {
    cfgs: Vec<GlobalCfg>,
}

impl GlobalCfgs {
    pub fn parse_attrs(attrs: &[Attribute]) -> Self {
        todo!()
    }
}