
use std::ffi::OsStr;
use std::os::unix::ffi::OsStrExt;

fn strip_pre<'a>(str: &'a OsStr, prefix: &str) -> Option<&'a OsStr> {

    let enc = str.as_bytes();
    let pre = prefix.as_bytes();

    enc.strip_prefix(pre)
        .and_then(|v| Some(OsStr::from_bytes(v)))
}


fn split_once(str: &OsStr, ch: char) -> Option<(&OsStr, &OsStr)> {
    let enc = str.as_bytes();
    let mut buf = [0; 1];
    let sep = ch.encode_utf8(&mut buf).as_bytes();

    enc.iter()
        .enumerate()
        .find(|(_, ch)| ch == &&sep[0])
        .and_then(|(idx, _)| {
            Some((
                OsStr::from_bytes(&enc[0..idx]),
                OsStr::from_bytes(&enc[idx + 1..]),
            ))
        })
}

pub trait AOsStrExt {
    fn strip_pre(&self, prefix: &str) -> Option<&OsStr>;

    fn split_once(&self, ch: char) -> Option<(&OsStr, &OsStr)>;
}

impl AOsStrExt for OsStr {
    fn strip_pre(&self, prefix: &str) -> Option<&OsStr> {
        strip_pre(self, prefix)
    }

    fn split_once(&self, ch: char) -> Option<(&OsStr, &OsStr)> {
        split_once(self, ch)
    }
}