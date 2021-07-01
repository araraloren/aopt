
use std::mem::take;

use super::*;
use crate::uid::Uid;
use crate::set::info::CreateInfo;

pub fn current_type() -> &'static str {
    "b"
}

pub trait Bool: Opt { }

pub struct BoolOpt {
    uid: Uid,

    name: String,

    prefix: String,

    optional: bool,

    value: OptValue,

    default_value: OptValue,

    deactivate_style: bool,

    alias: Vec<(String, String)>,

    callback_type: CallbackType,

    help: HelpInfo,
}

impl From<CreateInfo> for BoolOpt {
    fn from(ci: CreateInfo) -> Self {
        let mut ci = ci;
        
        Self {
            uid: ci.get_uid(),
            name: take(ci.get_name_mut()),
            prefix: take(ci.get_prefix_mut()),
            optional: ci.get_optional(),
            value: OptValue::default(),
            default_value: take(ci.get_default_value_mut()),
            deactivate_style: ci.get_deactivate_style(),
            alias: take(ci.get_alias_mut()),
            callback_type: ci.get_callback_type().clone(),
            help: take(ci.get_help_info_mut()),
        }
    }
}