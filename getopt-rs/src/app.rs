use crate::parser::Parser;
use crate::set::Set;

#[derive(Debug, Default)]
pub struct SingleApp<S: Set, P: Parser> {
    name: String,
    set: S,
    parser: P,
}
