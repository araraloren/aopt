use std::ops::Deref;

use quote::ToTokens;
use syn::parse::Parse;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::Attribute;
use syn::Token;

use crate::error;
use crate::value::Value;

pub mod arg;
pub mod cote;
pub mod fetch;
pub mod infer;
pub mod sub;
pub mod value;

pub use self::arg::ArgKind;
pub use self::cote::CoteKind;
pub use self::fetch::FetchKind;
pub use self::infer::InferKind;
pub use self::sub::SubKind;
pub use self::value::ValueKind;

#[derive(Debug)]
pub enum Style {
    Value,
    Flag,
    True,
}

pub trait Kind
where
    Self: Sized,
{
    fn parse(input: &mut ParseStream) -> syn::Result<(Self, Style)>;
}

#[derive(Debug, Clone)]
pub struct Config<T> {
    kind: T,

    value: Value,
}

impl<T> Config<T> {
    pub fn kind(&self) -> &T {
        &self.kind
    }

    pub fn value(&self) -> &Value {
        &self.value
    }
}

impl<T: Kind> Parse for Config<T> {
    fn parse(mut input: ParseStream) -> syn::Result<Self> {
        let (kind, style) = T::parse(&mut input)?;

        Ok(Self {
            kind,
            value: match style {
                Style::Value => input.parse()?,
                Style::Flag => Value::Null,
                Style::True => {
                    input
                        .parse()
                        .unwrap_or(Value::Literal(syn::Lit::Bool(syn::LitBool {
                            value: true,
                            span: input.span(),
                        })))
                }
            },
        })
    }
}

impl<T> ToTokens for Config<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        ToTokens::to_tokens(&self.value, tokens)
    }
}

#[derive(Debug, Clone)]
pub struct Configs<T>(Vec<Config<T>>);

impl<T> Configs<T>
where
    T: PartialEq,
{
    pub fn has_cfg(&self, kind: T) -> bool {
        self.0.iter().any(|v| v.kind() == &kind)
    }

    pub fn find_cfg(&self, kind: T) -> Option<&Config<T>> {
        self.0.iter().find(|v| v.kind() == &kind)
    }

    // pub fn find_cfgs(&self, kind: T) -> Vec<&Config<T>> {
    //     self.0.iter().filter(|v| v.kind() == &kind).collect()
    // }

    pub fn find_value(&self, kind: T) -> Option<&Value> {
        self.0.iter().find(|v| v.kind() == &kind).map(|v| v.value())
    }

    pub fn find_values(&self, kind: T) -> Vec<&Value> {
        self.0
            .iter()
            .filter(|v| v.kind() == &kind)
            .map(|v| v.value())
            .collect()
    }
}

impl<T: Kind> Configs<T> {
    pub fn parse_attrs(name: &str, attrs: &[Attribute]) -> Self {
        let attrs = attrs.iter().filter(|v| v.path().is_ident(name));
        let cfgs = attrs.map(|attr| {
            attr.parse_args_with(Punctuated::<Config<T>, Token![,]>::parse_terminated)
                .map(|res| res.into_iter())
                .unwrap_or_else(|e| {
                    panic!(
                        "{:?}",
                        error(
                            attr.span(),
                            format!("can not parsing `{}` attributes: {:?}", name, e),
                        )
                    )
                })
        });

        Self(cfgs.flatten().collect::<Vec<Config<T>>>())
    }
}

impl<T> Deref for Configs<T> {
    type Target = Vec<Config<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// pub fn find_cfg_name<'a>(names: &[&'a str], attrs: &[Attribute]) -> Option<&'a str> {
//     for attr in attrs {
//         for name in names {
//             if attr.path.is_ident(*name) {
//                 return Some(*name);
//             }
//         }
//     }
//     None
// }
