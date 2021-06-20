
pub mod arg;
pub mod str;
pub mod err;

pub(crate) mod pattern;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
