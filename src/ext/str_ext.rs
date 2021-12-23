use crate::*;



#[allow(dead_code)]
pub(crate) trait StrExt {
    fn as_str(&self) -> &str;

    fn find_token(&self, token: &str) -> Option<usize> {
        if token.is_empty() { debug_assert!(false, "token.is_empty()"); return None }

        let str = self.as_str();
        let mut start = 0;
        while let Some(idx) = str[start..].find(token).map(|i| start+i) {
            if str.is_ascii_word_boundary(idx) && str.is_ascii_word_boundary(idx+token.len()) {
                return Some(idx);
            }
            start = idx + token.len();
        }
        None
    }

    fn starts_with_token(&self, s: &str) -> bool {
        let str = self.as_str();
        str.starts_with(s) && str.is_ascii_word_boundary(s.len())
    }

    fn is_ascii_word_boundary(&self, idx: usize) -> bool {
        let bytes = self.as_str().as_bytes();
        idx == 0 || idx == bytes.len() || (bytes[idx-1].is_ascii_word_character() != bytes[idx-0].is_ascii_word_character())
    }

    fn strip_prefix_suffix  (&self, prefix: &str, suffix: &str) -> Option<&str> { self.as_str().strip_prefix(prefix).and_then(move |s| s.strip_suffix(suffix)) }
    fn split_once_trim      (&self, s: &str)            -> Option<(&str, &str)> { self.as_str().split_once(s).map(|(a, b)| (a.trim_end(), b.trim_start())) }
    fn rsplit_once_trim     (&self, s: &str)            -> Option<(&str, &str)> { self.as_str().rsplit_once(s).map(|(a, b)| (a.trim_end(), b.trim_start())) }
    fn try_split_once_trim  (&self, s: &str)            -> (&str, Option<&str>) { self.split_once_trim(s).map_or((self.as_str(), None), |(a,b)| (a, Some(b))) }
    fn try_rsplit_once_trim (&self, s: &str)            -> (Option<&str>, &str) { self.rsplit_once_trim(s).map_or((None, self.as_str()), |(a,b)| (Some(a), b)) }
}

impl StrExt for str {
    fn as_str(&self) -> &str { self }
}

#[test] fn test_word_boundary() {
    for (text,          bounds      ) in [
        ("FOO",         "w  w"      ),
        (" FOO ",       "ww  ww"    ),
        (" FOO BAR ",   "ww  ww  ww"),
    ].into_iter() {
        for (idx, expect_bound) in bounds.chars().map(|ch| ch != ' ').enumerate() {
            let is_bound = text.is_ascii_word_boundary(idx);
            assert!(
                is_bound == expect_bound,
                concat!(
                    "\n    text         = {:?}",
                    "\n    idx          = {}",
                    "\n    expect_bound = {}",
                    "\n    is_bound     = {}",
                ),
                text,
                idx,
                expect_bound,
                is_bound,
            );
        }
    }
}

#[test] fn test_find_token() {
    assert_eq!(Some(0),  "FOO" .find_token("FOO"));
    assert_eq!(Some(0),  "FOO ".find_token("FOO"));
    assert_eq!(Some(1), " FOO" .find_token("FOO"));
    assert_eq!(Some(1), " FOO ".find_token("FOO"));
    assert_eq!(None,    "xFOO ".find_token("FOO"));
    assert_eq!(None,    " FOOx".find_token("FOO"));
    assert_eq!(None,    "xFOOx".find_token("FOO"));
}
