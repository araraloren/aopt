
pub mod creator;
pub mod filter;

use std::fmt::Debug;

use crate::proc::Subscriber;
use creator::{ Creator, CreateInfo };

pub trait Set: Debug {
    fn add_creator(&mut self, creator: Box<dyn Creator>) -> bool;

    fn app_creator(&mut self, creator: Vec<Box<dyn Creator>>) -> bool;

    fn rem_creator(&mut self, opt_type: &str) -> bool;

    fn get_creator(&self, opt_type: &str) -> Option<&dyn Creator>;

    fn add_opt(&mut self, opt_str: &str) -> Result<Commit>;
}

impl<T: Set> Subscriber for T {
    fn subscribe_from(&self, publisher: &mut dyn crate::proc::Publisher<dyn crate::proc::Proc>) {
        
    }
}
