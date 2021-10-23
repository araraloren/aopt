
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Index {
    Forward(u64),

    Backward(u64),

    List(Vec<u64>),

    Except(Vec<u64>),

    Greater(u64),

    Less(u64),

    AnyWhere,

    Null,
}

impl Index {
    pub fn forward(index: u64) -> Self {
        Self::Forward(index)
    }

    pub fn backward(index: u64) -> Self {
        Self::Backward(index)
    }

    pub fn list(list: Vec<u64>) -> Self {
        Self::List(list)
    }

    pub fn except(list: Vec<u64>) -> Self {
        Self::Except(list)
    }

    pub fn greater(index: u64) -> Self {
        Self::Greater(index)
    }

    pub fn less(index: u64) -> Self {
        Self::Less(index)
    }

    pub fn anywhere() -> Self {
        Self::AnyWhere
    }

    pub fn null() -> Self {
        Self::Null
    }

    pub fn is_null(&self) -> bool {
        match self {
            Self::Null => true,
            _ => false,
        }
    }

    pub fn calc_index(&self, total: u64, current: u64) -> Option<u64> {
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

                    if offset <= total && offset == current {
                        return Some(offset);
                    }
                }
            }
            Self::Except(list) => {
                for offset in list {
                    let offset = *offset;

                    if offset <= total && offset != current {
                        return Some(current);
                    }
                }
            }
            Self::Greater(offset) => {
                let offset = *offset;

                if offset <= total && offset < current {
                    return Some(current);
                }
            }
            Self::Less(offset) => {
                let offset = *offset;

                if offset <= total && offset > current {
                    return Some(current);
                }
            }
            Self::AnyWhere => {
                return Some(current);
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
            Index::Forward(v) | Index::Backward(v) => {
                format!("{}", v)
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
            Index::AnyWhere => {
                format!("*")
            }
            Index::Null => String::default(),
        }
    }
}
