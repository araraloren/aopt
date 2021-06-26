
use crate::opt::{
    Opt,
    index::Index,
    value::Value, 
    callback::CallbackType, 
    help::HelpInfo
};

#[derive(Debug, Clone, Default)]
pub struct Builder {
    deactivate_style: bool,

    optional: bool,

    type_name: String,

    name: String,

    prefix: String,

    index: Index,

    value: Value,

    alias: Vec<(String, String)>,

    callback_type: CallbackType,

    help: HelpInfo,
}

impl Builder {
    
}