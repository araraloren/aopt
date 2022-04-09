/// Index using for [`NonOpt`](crate::opt::NonOpt).
///
/// The [`NonOpt`](crate::opt::NonOpt) index is the position of non-option-argument(NOA) index, its base on 1.
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    Forward(usize),

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

    /// The NOA which index bigger than given position.
    ///
    /// # Example
    ///
    /// For `["--aopt", "--bopt=42", "pos1", "--copt", "pos2", "--dopt", "value", "pos3"]`:
    ///
    /// `@>0` will matching `"pos1"`, `"pos2"` or `"pos3"`.
    ///
    /// `@>2` will matching `"pos3"`.
    ///
    /// `@>1` will matching `"pos2"` or `"pos3"`.
    Greater(usize),

    /// The NOA which index little than given position.
    ///
    /// # Example
    ///
    /// For `["--aopt", "--bopt=42", "pos1", "--copt", "pos2", "--dopt", "value", "pos3"]`:
    ///
    /// `@<4` will matching `"pos1"`, `"pos2"` or `"pos3"`.
    ///
    /// `@<2` will matching `"pos1"`.
    Less(usize),

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
    pub fn forward(index: usize) -> Self {
        Self::Forward(index)
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

    pub fn greater(index: usize) -> Self {
        Self::Greater(index)
    }

    pub fn less(index: usize) -> Self {
        Self::Less(index)
    }

    pub fn anywhere() -> Self {
        Self::AnyWhere
    }

    pub fn null() -> Self {
        Self::Null
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    /// Compare the NOA information with current Index.
    pub fn calc_index(&self, index: usize, total: usize) -> Option<usize> {
        match self {
            Self::Forward(offset) => {
                let offset = *offset;

                if offset <= total {
                    return Some(offset);
                }
            }
            Self::Backward(offset) => {
                let offset = *offset;

                if offset <= total {
                    return Some(total - offset + 1);
                }
            }
            Self::List(list) => {
                for offset in list {
                    let offset = *offset;

                    if offset <= total && offset == index {
                        return Some(offset);
                    }
                }
            }
            Self::Except(list) => {
                if index <= total && !list.contains(&index) {
                    return Some(index);
                }
            }
            Self::Greater(offset) => {
                let offset = *offset;

                if offset <= total && offset < index {
                    return Some(index);
                }
            }
            Self::Less(offset) => {
                let offset = *offset;

                if offset <= total && offset > index {
                    return Some(index);
                }
            }
            Self::AnyWhere => {
                return Some(index);
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
            Index::Forward(v) => {
                format!("{}", v)
            }
            Index::Backward(v) => {
                format!("-{}", v)
            }
            Index::List(v) => {
                let strs: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();

                format!("[{}]", strs.join(", "))
            }
            Index::Except(v) => {
                let strs: Vec<String> = v.iter().map(|v| format!("{}", v)).collect();

                format!("-[{}]", strs.join(", "))
            }
            Index::Greater(v) => {
                format!(">{}", v)
            }
            Index::Less(v) => {
                format!("<{}", v)
            }
            Index::AnyWhere => "*".to_string(),
            Index::Null => String::default(),
        }
    }
}
