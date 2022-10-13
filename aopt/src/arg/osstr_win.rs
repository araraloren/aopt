use std::ffi::OsStr;
use std::ffi::OsString;
use std::os::windows::ffi::{OsStrExt, OsStringExt};

fn strip_pre(str: &OsStr, prefix: &str) -> Option<OsString> {
    let enc = str.encode_wide();
    let mut pre = prefix.encode_utf16();
    let mut ret = Vec::with_capacity(str.len() - prefix.len());

    for ori in enc {
        match pre.next() {
            Some(ch) => {
                if ch != ori {
                    return None;
                }
            }
            None => {
                ret.push(ori);
            }
        }
    }
    Some(OsString::from_wide(&ret))
}

fn split_once(str: &OsStr, ch: char) -> Option<(OsString, OsString)> {
    let enc = str.encode_wide();
    let mut buf = [0; 1];
    let sep = ch.encode_utf16(&mut buf);
    let enc = enc.collect::<Vec<u16>>();

    enc.iter()
        .enumerate()
        .find(|(_, ch)| ch == &&sep[0])
        .and_then(|(idx, _)| {
            Some((
                OsString::from_wide(&enc[0..idx]),
                OsString::from_wide(&enc[idx + 1..]),
            ))
        })
}

pub trait AOsStrExt {
    fn strip_pre(&self, prefix: &str) -> Option<OsString>;

    fn split_once(&self, ch: char) -> Option<(OsString, OsString)>;
}

impl AOsStrExt for OsStr {
    fn strip_pre(&self, prefix: &str) -> Option<OsString> {
        strip_pre(self, prefix)
    }

    fn split_once(&self, ch: char) -> Option<(OsString, OsString)> {
        split_once(self, ch)
    }
}