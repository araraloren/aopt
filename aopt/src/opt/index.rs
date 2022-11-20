/// Index using for option match.
///
/// The index is the position of left arguments (non-option arguments, NOA) index, its base on 1.
///
/// # Example
///
/// ```txt
/// foo.exe -a=value -b value pos1 --aopt=42 pos2 --bopt value pos3
///             |     |   |    |      |       |      |     |     |
///             |     |   |    |      |       |      |     |     NOA @3 or @-1
///             |     |   |    |      |       |      |     value of --bopt
///             |     |   |    |      |       |      option --bopt
///             |     |   |    |      |       NOA @2 or @-2
///             |     |   |    |    option --aopt and its value 42
///             |     |   |   NOA @1
///             |     |   value of -b
///             |    option -b
///         option -a and its value
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Index {
    /// The forward index of NOA.
    ///
    /// # Example
    ///
    /// For `["--aopt", "--bopt=42", "pos1", "--copt", "pos2", "--dopt", "value", "pos3"]`:
    ///
    /// `@1` will matching `"pos1"`.
    ///
    /// `@2` will matching `"pos2"`.
    ///
    /// `@3` will matching `"pos3"`.
    FwdPolicy(usize),

    /// The backward index of NOA.
    ///
    /// # Example
    ///
    /// For `["--aopt", "--bopt=42", "pos1", "--copt", "pos2", "--dopt", "value", "pos3"]`:
    ///
    /// `@-1` will matching `"pos3"`.
    ///
    /// `@-2` will matching `"pos2"`.
    ///
    /// `@-3` will matching `"pos1"`.
    Backward(usize),

    /// The include list of forward index of NOA.
    ///
    /// # Example
    ///
    /// For `["--aopt", "--bopt=42", "pos1", "--copt", "pos2", "--dopt", "value", "pos3"]`:
    ///
    /// `@[1,3]` will matching `"pos1"` or `"pos3"`.
    ///
    /// `@[1,2]` will matching `"pos1"` or `"pos2"`.
    ///
    /// `@[1,2,3]` will matching `"pos1"`, `"pos2"` or `"pos3"`.
    List(Vec<usize>),

    /// The exclude list of forward index of NOA.
    ///
    /// # Example
    ///
    /// For `["--aopt", "--bopt=42", "pos1", "--copt", "pos2", "--dopt", "value", "pos3"]`:
    ///
    /// `@-[1,3]` will matching `"pos2"`.
    ///
    /// `@-[3]` will matching `"pos1"` or `"pos2"`.
    ///
    /// `@-[2]` will matching `"pos1"` or `"pos3"`.
    Except(Vec<usize>),

    /// The NOA which index inside in given position range.
    ///
    /// # Example
    ///
    /// For `["--aopt", "--bopt=42", "pos1", "--copt", "pos2", "--dopt", "value", "pos3"]`:
    ///
    /// `@0..` will matching `"pos1"`, `"pos2"` or `"pos3"`.
    ///
    /// `@2..` will matching `"pos3"`.
    ///
    /// `@1..` will matching `"pos2"` or `"pos3"`.
    ///
    /// `@..4` will matching `"pos1"`, `"pos2"` or `"pos3"`.
    ///
    /// `@..2` will matching `"pos1"`.
    ///
    /// `@1..3` will matching `"pos1"`, `"pos2"`.
    Range(usize, usize),

    /// The anywhere position of NOA.
    ///
    /// # Example
    ///
    /// For `["--aopt", "--bopt=42", "pos1", "--copt", "pos2", "--dopt", "value", "pos3"]`:
    ///
    /// `@*` or `@0` will matching `"pos1"`, `"pos2"` or `"pos3"`.
    AnyWhere,

    Null,
}

impl Index {
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub fn is_forward(&self) -> bool {
        matches!(self, Self::FwdPolicy(_))
    }

    pub fn is_backward(&self) -> bool {
        matches!(self, Self::Backward(_))
    }

    pub fn is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }

    pub fn is_except(&self) -> bool {
        matches!(self, Self::Except(_))
    }

    pub fn is_range(&self) -> bool {
        matches!(self, Self::Range(_, _))
    }

    pub fn is_anywhere(&self) -> bool {
        matches!(self, Self::AnyWhere)
    }

    pub fn to_help(&self) -> String {
        String::default()
    }

    pub fn forward(index: usize) -> Self {
        Self::FwdPolicy(index)
    }

    pub fn backward(index: usize) -> Self {
        Self::Backward(index)
    }

    pub fn list(list: Vec<usize>) -> Self {
        Self::List(list)
    }

    pub fn except(list: Vec<usize>) -> Self {
        Self::Except(list)
    }

    pub fn range(start: Option<usize>, end: Option<usize>) -> Self {
        match (start, end) {
            (None, None) => {
                panic!("start and end can't both None")
            }
            (None, Some(end)) => Self::Range(0, end),
            (Some(start), None) => Self::Range(start, 0),
            (Some(start), Some(end)) => Self::Range(start, end),
        }
    }

    pub fn anywhere() -> Self {
        Self::AnyWhere
    }

    pub fn null() -> Self {
        Self::Null
    }

    pub fn calc_index(&self, noa_index: usize, noa_count: usize) -> Option<usize> {
        match self {
            Self::FwdPolicy(offset) => {
                let offset = *offset;

                if offset <= noa_count {
                    return Some(offset);
                }
            }
            Self::Backward(offset) => {
                let offset = *offset;

                if offset <= noa_count {
                    return Some(noa_count - offset + 1);
                }
            }
            Self::List(list) => {
                for offset in list {
                    let offset = *offset;

                    if offset <= noa_count && offset == noa_index {
                        return Some(offset);
                    }
                }
            }
            Self::Except(list) => {
                if noa_index <= noa_count && !list.contains(&noa_index) {
                    return Some(noa_index);
                }
            }
            Self::Range(start, end) => match (start, end) {
                (0, end) => {
                    let end = *end;

                    if noa_index < end {
                        return Some(noa_index);
                    }
                }
                (start, 0) => {
                    let start = *start;

                    if noa_index >= start {
                        return Some(noa_index);
                    }
                }
                (start, end) => {
                    let start = *start;
                    let end = *end;

                    if noa_index >= start && noa_index < end {
                        return Some(noa_index);
                    }
                }
            },
            Self::AnyWhere => {
                return Some(noa_index);
            }
            _ => {}
        }
        None
    }
}

impl Default for Index {
    fn default() -> Self {
        Self::Null
    }
}

impl ToString for Index {
    fn to_string(&self) -> String {
        match self {
            Index::AnyWhere => "*".to_string(),
            Index::Null => String::default(),
            Index::FwdPolicy(v) => {
                format!("{}", v)
            }
            Index::Backward(v) => {
                format!("-{}", v)
            }
            Index::Range(0, e) => {
                format!("..{}", e)
            }
            Index::Range(s, 0) => {
                format!("{}..", s,)
            }
            Index::Range(s, e) => {
                format!("{}..{}", s, e)
            }
            Index::List(v) => {
                let strs: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();

                format!("[{}]", strs.join(", "))
            }
            Index::Except(v) => {
                let strs: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();

                format!("-[{}]", strs.join(", "))
            }
        }
    }
}
