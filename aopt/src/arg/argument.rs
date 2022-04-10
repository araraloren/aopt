use crate::err::Error;
use crate::err::Result;
use regex::Regex;
use ustr::Ustr;

#[derive(Debug, Clone, Default)]
pub struct DataKeeper {
    pub name: Option<Ustr>,

    pub value: Option<Ustr>,

    pub prefix: Option<Ustr>,

    pub disable: bool,
}

/// Argument hold current and next item of command line arguments.
///
/// When parsing the command line option need an argument.
/// The argument of option may embedded in itself.
/// Or we need consume next item as argument of the option.
#[derive(Debug, Clone, Default)]
pub struct Argument {
    pub current: Option<Ustr>,

    pub next: Option<Ustr>,

    data_keeper: DataKeeper,
}

impl Argument {
    pub fn new(current: Option<Ustr>, next: Option<Ustr>) -> Self {
        Self {
            current,
            next,
            ..Self::default()
        }
    }

    pub fn get_data_keeper(&self) -> &DataKeeper {
        &self.data_keeper
    }

    pub fn get_prefix(&self) -> &Option<Ustr> {
        &self.data_keeper.prefix
    }

    pub fn get_name(&self) -> &Option<Ustr> {
        &self.data_keeper.name
    }

    pub fn get_value(&self) -> &Option<Ustr> {
        &self.data_keeper.value
    }

    /// Return true if the option contain deactivate style symbol '/'
    pub fn is_disabled(&self) -> bool {
        self.data_keeper.disable
    }

    /// Call [`parse_argument`] parse the command line item with given regexs.
    ///
    /// # Returns
    ///
    /// Will save the [`DataKeeper`] to self and return `Ok(true)` when successed.
    /// Return `Ok(false)` when current item is [`None`].
    ///
    /// # Errors
    ///
    /// - [`ArgumentError::MissingPrefix`](crate::err::ArgumentError::MissingPrefix)
    ///
    /// When the result not have a valid prefix.
    ///
    /// - [`ArgumentError::MissingName`](crate::err::ArgumentError::MissingName)
    ///
    /// When the result not have a valid name.
    pub fn parse(&mut self, regexs: &[Regex]) -> Result<bool> {
        if let Some(pattern) = &self.current {
            self.data_keeper = parse_argument(*pattern, regexs)?;

            // must have prefix and name
            if self.data_keeper.prefix.is_none() {
                return Err(Error::arg_missing_prefix(pattern));
            }
            if self.data_keeper.name.is_none() {
                return Err(Error::arg_missing_name(pattern));
            }
            return Ok(true);
        }
        Ok(false)
    }
}

/// Parse the input command line item with given regexs, return an [`DataKeeper`].
///
/// The struct of the input option string are:
///
/// ```!
/// [--][/][option][=][value]
///   |  |     |    |    |
///   |  |     |    |    |
///   |  |     |    |    The value part, it is optional.
///   |  |     |    |
///   |  |     |    The delimiter of option name and value.
///   |  |     |    
///   |  |     The option name part, it must be provide by user.
///   |  |
///   |  The disable symbol, generally it is using for boolean option.
///   |  
///   The prefix of option.
/// ```
///
/// # Example
///
/// ```rust
/// use aopt::arg::parse_argument;
/// use aopt::arg::create_prefix_regexs;
/// use ustr::Ustr;
/// use aopt::gstr;
/// use aopt::err::Result;
///
/// fn main() -> Result<()> {
///     let regexs = create_prefix_regexs(&[gstr("--"), gstr("-")])?;
///
///     {// parse option with value
///         let dk = parse_argument(gstr("--foo=32"), &regexs)?;
///
///         assert_eq!(dk.prefix, Some(gstr("--")));
///         assert_eq!(dk.name, Some(gstr("foo")));
///         assert_eq!(dk.value, Some(gstr("32")));
///         assert_eq!(dk.disable, false);
///     }
///     {// parse boolean option
///         let dk = parse_argument(gstr("--/bar"), &regexs)?;
///
///         assert_eq!(dk.prefix, Some(gstr("--")));
///         assert_eq!(dk.name, Some(gstr("bar")));
///         assert_eq!(dk.value, None);
///         assert_eq!(dk.disable, true);
///     }
///     {// parse other string
///         let dk = parse_argument(gstr("-=bar"), &regexs);
///
///         assert!(dk.is_err());
///     }
///     Ok(())
/// }
/// ```
pub fn parse_argument(pattern: Ustr, regexs: &[Regex]) -> Result<DataKeeper> {
    for regex in regexs {
        if let Some(capture) = regex.captures(pattern.as_str()) {
            return Ok(DataKeeper {
                name: capture.get(3).map(|v| v.as_str().into()),
                prefix: capture.get(1).map(|v| v.as_str().into()),
                value: capture.get(5).map(|v| v.as_str().into()),
                disable: capture.get(2).is_some(),
            });
        }
    }
    Err(Error::arg_parsing_failed(pattern))
}

pub fn create_prefix_regexs(prefix: &[Ustr]) -> Result<Vec<Regex>> {
    let mut ret = vec![];

    for prefix in prefix {
        ret.push(
            Regex::new(&format!(
                "({})(/)?([^=]+)(=(.+))?",
                regex::escape(prefix.as_str())
            ))
            .map_err(|_| Error::opt_invalid_prefix(prefix))?,
        );
    }
    Ok(ret)
}

#[cfg(test)]
mod test {
    use regex::Regex;
    use ustr::Ustr;

    use crate::arg::create_prefix_regexs;
    use crate::arg::parse_argument;
    use crate::gstr;

    #[test]
    fn test_for_input_parser() {
        {
            // test 1
            let test_cases = vec![
                ("", None),
                ("-a", Some((Some("-"), Some("a"), None, false))),
                ("-/a", Some((Some("-"), Some("a"), None, true))),
                ("-a=b", Some((Some("-"), Some("a"), Some("b"), false))),
                ("--foo", Some((Some("--"), Some("foo"), None, false))),
                ("--/foo", Some((Some("--"), Some("foo"), None, true))),
                (
                    "--foo=bar",
                    Some((Some("--"), Some("foo"), Some("bar"), false)),
                ),
                ("a", Some((Some(""), Some("a"), None, false))),
                ("/a", Some((Some(""), Some("a"), None, true))),
                ("a=b", Some((Some(""), Some("a"), Some("b"), false))),
                ("foo", Some((Some(""), Some("foo"), None, false))),
                ("/foo", Some((Some(""), Some("foo"), None, true))),
                ("foo=bar", Some((Some(""), Some("foo"), Some("bar"), false))),
                ("--=xar", Some((Some("-"), Some("-"), Some("xar"), false))),
                ("-foo=", Some((Some("-"), Some("foo"), None, false))),
            ];

            let regexs = create_prefix_regexs(&vec![gstr("--"), gstr("-"), gstr("")]).unwrap();

            for case in test_cases.iter() {
                try_to_verify_one_task(gstr(case.0), &regexs, case.1);
            }
        }
        {
            // test 2
            let test_cases = vec![
                ("", None),
                ("-a", Some((Some("-"), Some("a"), None, false))),
                ("-/a", Some((Some("-"), Some("a"), None, true))),
                ("-a=b", Some((Some("-"), Some("a"), Some("b"), false))),
                ("--foo", Some((Some("--"), Some("foo"), None, false))),
                ("--/foo", Some((Some("--"), Some("foo"), None, true))),
                (
                    "--foo=bar",
                    Some((Some("--"), Some("foo"), Some("bar"), false)),
                ),
                ("a", None),
                ("/a", None),
                ("a=b", None),
                ("foo", None),
                ("/foo", None),
                ("foo=bar", None),
                ("--=xar", Some((Some("-"), Some("-"), Some("xar"), false))),
                ("-foo=", Some((Some("-"), Some("foo"), None, false))),
            ];

            let regexs = create_prefix_regexs(&vec![gstr("--"), gstr("-")]).unwrap();

            for case in test_cases.iter() {
                try_to_verify_one_task(gstr(case.0), &regexs, case.1);
            }
        }
        {
            // test 3
            let test_cases = vec![
                ("", None),
                ("+a", Some((Some("+"), Some("a"), None, false))),
                ("+/a", Some((Some("+"), Some("a"), None, true))),
                ("+a=b", Some((Some("+"), Some("a"), Some("b"), false))),
                ("++foo", Some((Some("++"), Some("foo"), None, false))),
                ("++/foo", Some((Some("++"), Some("foo"), None, true))),
                (
                    "++foo=bar",
                    Some((Some("++"), Some("foo"), Some("bar"), false)),
                ),
                ("a", None),
                ("/a", None),
                ("a=b", None),
                ("foo", None),
                ("/foo", None),
                ("foo=bar", None),
                ("++=xar", Some((Some("+"), Some("+"), Some("xar"), false))),
                ("+foo=", Some((Some("+"), Some("foo"), None, false))),
            ];

            let regexs = create_prefix_regexs(&vec![gstr("++"), gstr("+")]).unwrap();

            for case in test_cases.iter() {
                try_to_verify_one_task(gstr(case.0), &regexs, case.1);
            }
        }
    }

    fn try_to_verify_one_task(
        pattern: Ustr,
        regexs: &Vec<Regex>,
        except: Option<(Option<&str>, Option<&str>, Option<&str>, bool)>,
    ) {
        let ret = parse_argument(pattern, regexs);

        if let Ok(dk) = ret {
            assert!(except.is_some());

            if except.is_none() {
                panic!("----> {:?} {:?}", except, &dk);
            }

            let default = gstr("");

            if let Some(except) = except {
                assert_eq!(
                    except.0.unwrap_or(""),
                    dk.prefix.unwrap_or(default).as_ref()
                );
                assert_eq!(except.1.unwrap_or(""), dk.name.unwrap_or(default).as_ref());
                assert_eq!(except.2.unwrap_or(""), dk.value.unwrap_or(default).as_ref());
                assert_eq!(except.3, dk.disable);
            }
        } else {
            assert!(except.is_none());
        }
    }
}
