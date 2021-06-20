
use supercow::Supercow;

/// All the inner struct will using this type
/// hold the string value .
/// 
/// ```no_run
/// fn cow_string_example() {
///    #[derive(Debug)]
///    struct Holder<'a> {
///        data: Str<'a>,
///    }
///
///    impl<'a, 'b: 'a> Holder<'a> {
///        pub fn new(data: &'b str) -> Self {
///            Self {
///                data: Str::borrowed(data),
///            }
///        }
///    }
///
///    let s = String::from("inner_data");
///
///    dbg!(Holder::new(s.as_ref()));
/// }
/// ```
pub type Str<'a> = Supercow<'a, String, str>;

