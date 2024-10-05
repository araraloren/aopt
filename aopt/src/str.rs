use std::borrow::Cow;
use std::ffi::OsStr;
use std::ffi::OsString;

#[cfg(target_family = "windows")]
pub fn split_once(str: &OsStr, ch: char) -> Option<(Cow<'_, OsStr>, Cow<'_, OsStr>)> {
    use std::ffi::OsString;
    use std::os::windows::ffi::{OsStrExt, OsStringExt};

    let enc = str.encode_wide();
    let mut buf = [0; 1];
    let sep = ch.encode_utf16(&mut buf);
    let enc = enc.collect::<Vec<u16>>();

    enc.iter()
        .enumerate()
        .find(|(_, ch)| ch == &&sep[0])
        .map(|(i, _)| {
            (
                Cow::Owned(OsString::from_wide(&enc[0..i])),
                Cow::Owned(OsString::from_wide(&enc[i + 1..])),
            )
        })
}

#[cfg(any(target_family = "wasm", target_family = "unix"))]
pub fn split_once<'a>(str: &'a OsStr, ch: char) -> Option<(Cow<'a, OsStr>, Cow<'a, OsStr>)> {
    #[cfg(target_family = "unix")]
    use std::os::unix::ffi::OsStrExt;
    #[cfg(target_family = "wasm")]
    use std::os::wasi::ffi::OsStrExt;

    let enc = str.as_bytes();
    let mut buf = [0; 1];
    let sep = ch.encode_utf8(&mut buf).as_bytes();

    enc.iter()
        .enumerate()
        .find(|(_, ch)| ch == &&sep[0])
        .map(|(i, _)| {
            (
                Cow::Borrowed(OsStr::from_bytes(&enc[0..i])),
                Cow::Borrowed(OsStr::from_bytes(&enc[i + 1..])),
            )
        })
}

/// Convert a [`OsStr`] to [`Cow<'_, str>`].
pub fn osstr_to_str_i<'a>(val: &[&'a OsStr], i: usize) -> Option<Cow<'a, str>> {
    val.get(i).and_then(|v| v.to_str().map(Cow::Borrowed))
}

pub fn display_of_str(val: Option<&str>) -> String {
    if let Some(val) = val {
        format!("Some({})", val)
    } else {
        "None".to_string()
    }
}

pub fn display_of_osstr(val: Option<&OsStr>) -> String {
    if let Some(val) = val {
        format!("Some({})", std::path::Path::new(val).display())
    } else {
        "None".to_string()
    }
}

pub trait CowOsStrUtils<'a> {
    fn split_once(&self, sep: char) -> Option<(Cow<'a, OsStr>, Cow<'a, OsStr>)>;

    fn to_str(&self, func: impl Fn(&str) -> &str) -> Option<Cow<'a, str>>;
}

impl<'a> CowOsStrUtils<'a> for Cow<'a, OsStr> {
    fn split_once(&self, sep: char) -> Option<(Cow<'a, OsStr>, Cow<'a, OsStr>)> {
        match self {
            Cow::Borrowed(v) => split_once(v, sep),
            Cow::Owned(v) => split_once(v, sep)
                .map(|(a, b)| (Cow::Owned(a.into_owned()), Cow::Owned(b.into_owned()))),
        }
    }

    fn to_str(&self, func: impl Fn(&str) -> &str) -> Option<Cow<'a, str>> {
        match &self {
            Cow::Borrowed(v) => v.to_str().map(func).map(Cow::Borrowed),
            Cow::Owned(v) => v.to_str().map(func).map(String::from).map(Cow::Owned),
        }
    }
}

pub trait CowStrUtils<'a> {
    fn split_at(&self, mid: usize) -> (Cow<'a, str>, Cow<'a, str>);

    fn to_os_str(self) -> Cow<'a, OsStr>;
}

impl<'a> CowStrUtils<'a> for Cow<'a, str> {
    fn split_at(&self, mid: usize) -> (Cow<'a, str>, Cow<'a, str>) {
        match self {
            Cow::Borrowed(v) => {
                let (a, b) = v.split_at(mid);

                (Cow::Borrowed(a), Cow::Borrowed(b))
            }
            Cow::Owned(v) => {
                let (a, b) = v.split_at(mid);

                (Cow::Owned(a.to_string()), Cow::Owned(b.to_string()))
            }
        }
    }

    fn to_os_str(self) -> Cow<'a, OsStr> {
        match self {
            Cow::Borrowed(v) => Cow::Borrowed(OsStr::new(v)),
            Cow::Owned(v) => Cow::Owned(OsString::from(v)),
        }
    }
}
