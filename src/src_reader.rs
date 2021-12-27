use crate::*;

use mmrbi::*;

use std::fmt::{self, Debug, Display, Formatter};
use std::num::NonZeroUsize;
use std::ops::Deref;
use std::path::*;
use std::sync::*;



pub(crate) struct SrcReader<'t> {
    path:               Arc<Path>,
    eols:               Vec<usize>,
    full_source:        &'t str,
    cursor:             usize,
}

pub(crate) struct SrcLine<'t> {
    pub location:       Location,
    pub raw:            &'t str,
    pub trimmed:        &'t str,
}

#[derive(Clone, Copy)]
pub(crate) struct SrcToken<'t> {
    token:              &'t str,
    idx:                usize,
}

impl Deref for SrcReader<'_> { type Target = str; fn deref(&self) -> &Self::Target { self.full_source   } }
impl Deref for SrcLine  <'_> { type Target = str; fn deref(&self) -> &Self::Target { self.raw           } }
impl Deref for SrcToken <'_> { type Target = str; fn deref(&self) -> &Self::Target { self.token         } }

impl Debug for SrcLine <'_> { fn fmt(&self, fmt: &mut Formatter) -> fmt::Result { Debug::fmt(&**self, fmt) } }
impl Debug for SrcToken<'_> { fn fmt(&self, fmt: &mut Formatter) -> fmt::Result { write!(fmt, "`{}`", &**self) } }
impl Display for SrcLine <'_> { fn fmt(&self, fmt: &mut Formatter) -> fmt::Result { Display::fmt(&**self, fmt) } }
impl Display for SrcToken<'_> { fn fmt(&self, fmt: &mut Formatter) -> fmt::Result { Display::fmt(&**self, fmt) } }

impl PartialEq< str> for SrcToken<'_> { fn eq(&self, other: &str     ) -> bool { &**self ==  other } }
impl PartialEq<&str> for SrcToken<'_> { fn eq(&self, other: &&str    ) -> bool { &**self == *other } }
impl PartialEq<SrcToken<'_>> for  str { fn eq(&self, other: &SrcToken) -> bool { &**other ==  self } }
impl PartialEq<SrcToken<'_>> for &str { fn eq(&self, other: &SrcToken) -> bool { &**other == *self } }


impl<'t> SrcReader<'t> {
    pub fn new(path: Arc<Path>, source: &'t str) -> Self {
        Self {
            path,
            eols:               source.char_indices().filter(|(_, ch)| *ch == '\n').map(|(i, _)| i).chain(Some(source.len())).collect(),
            full_source:        source,
            cursor:             0,
        }
    }

    pub fn position(&self) -> usize { self.cursor }
    pub fn set_position(&mut self, cursor: usize) { self.cursor = cursor }
    pub fn reset(&mut self) { self.cursor = 0; }

    pub fn next_line(&mut self) -> Option<SrcLine<'t>> {
        let remaining_source = self.full_source.get(self.cursor..).unwrap_or("");
        if remaining_source.is_empty() { return None; }
        let eol = remaining_source.find('\n').unwrap_or(remaining_source.len());

        let raw         = remaining_source[..eol].trim_end_matches('\r');
        let trimmed     = raw.trim();
        let location    = self.idx_to_location(self.cursor);
        self.cursor     += eol + 1;

        Some( SrcLine { location, raw, trimmed } )
    }

    pub fn next_token(&mut self) -> Option<SrcToken<'t>> {
        let src = self.full_source.get(self.cursor..)?.trim_start_cpp_whitespace_comments();
        let first_char  = src.chars().next()?;
        self.cursor = self.full_source.len() - src.len();

        macro_rules! fail {
            ( $($tt:tt)* ) => {{
                let (line_no, col_no) = self.idx_to_line_col_no(self.cursor);
                warning!(at: &self.path, line: line_no, column: col_no, $($tt)*);
                return None; // give up on parsing
            }};
        }


        // https://en.cppreference.com/w/cpp/language/character_literal
        // https://en.cppreference.com/w/cpp/language/string_literal

        if "_uUL\'\"".contains(first_char) {
            for (pre,   raw,    post) in [
                ("",    false,  ""  ), // char
                ("L",   false,  ""  ), // wchar_t
                ("LR",  true,   ""  ), // wchar_t
                ("_T(", false,  ")" ), // TCHAR
                ("u8",  false,  ""  ), // char8_t
                ("u8R", true,   ""  ), // char8_t
                ("u",   false,  ""  ), // char16_t
                ("uR",  true,   ""  ), // char16_t
                ("U",   false,  ""  ), // char32_t
                ("UR",  true,   ""  ), // char32_t
            ].into_iter() {
                if !src.starts_with(pre) { continue }
                let mut chars = src[pre.len()..].chars();
                let quote = chars.next().unwrap_or(' ');
                if !"\'\"".contains(quote) { continue }

                //let (line_no, col_no) = self.idx_to_line_col_no(self.cursor);
                //info!(at: &self.path, line: line_no, column: col_no, "start of string: {}...", src.split_once('\n').map_or(src, |(before, _)| before).trim_end_matches("\r"));

                // OK, found a string

                if raw { fail!("raw strings not yet supported"); }
                while let Some(ch) = chars.next() {
                    if ch == '\\' {
                        let _ = chars.next(); // escape
                    } else if ch == quote {
                        // TODO: potentially collase multiple strings?
                        if !chars.as_str().starts_with(post) { fail!("expected {:?} after final quote of character/string", post); }
                        let token_len = src.len() - chars.as_str().len() + post.len();
                        let token = &src[..token_len];
                        let idx = self.cursor;
                        self.cursor += token_len;
                        return Some(SrcToken { token, idx });
                    } else {
                        // string char
                    }
                }
                fail!("unterminated string? {}", src.split_once('\n').map_or(src, |(before, _)| before).trim_end_matches("\r"));
            }
        }


        // https://en.cppreference.com/w/cpp/language/floating_literal
        // https://en.cppreference.com/w/cpp/language/integer_literal
        if first_char == '.' || first_char.is_ascii_digit() {
            let mut rest = src;
            let mut hex = false;

            // handle prefixes like "0x", "0b", etc.
            if first_char == '0' {
                rest = match rest.get(1..2).unwrap_or("") {
                    "x" | "X" => { hex = true;  &rest[2..] }, // hex
                    "b" | "B" => { hex = false; &rest[2..] }, // binary
                    d if d.starts_with(|ch: char| ch.is_ascii_digit()) => { // octal(ish)
                        hex = false;
                        &rest[1..]
                    },
                    _ => rest, // decimal
                };
            }

            // handle integer, fraction, and exponent
            while let Some(ch) = rest.chars().next() {
                if ch == '\'' {
                    // digit separator
                    rest = &rest[1..];
                } else if ch == '.' {
                    // decimal separator
                    rest = &rest[1..];
                } else if ch == 'p' || (ch == 'e' && !hex) {
                    // exponent
                    rest = &rest[1..];
                    // optional sign
                    if "+-".contains(rest.chars().next().unwrap_or(' ')) { rest = &rest[1..]; }
                    // exponent digits
                    rest = rest.trim_start_matches(|ch: char| ch.is_ascii_hexdigit());
                    break;
                } else if ch.is_ascii_hexdigit() {
                    // digit
                    rest = &rest[1..];
                } else {
                    // not part of identifier
                    break;
                }
            }

            // handle type suffixes like "ul", "f", "ULL", "zu", etc.
            rest = rest.trim_start_matches(|ch: char| ch.is_ascii_alphabetic());

            let idx = self.cursor;
            let end = src.len() - rest.len();
            self.cursor += end;
            return Some(SrcToken { token: &src[..end], idx });
        }


        // idents
        if first_char.is_ascii_word_character() {
            let idx = self.cursor;
            let end = src.find(|ch: char| !ch.is_ascii_word_character()).unwrap_or(src.len());
            self.cursor += end;
            return Some(SrcToken { token: &src[..end], idx });
        }


        for op in [
            // N.B. longest first!
            // https://en.cppreference.com/w/cpp/language/operators
            "<<= >>= <=> ->* ...",      // 3
            "== != <= >= << >> ++ --",  // 2
            "+= -= *= /= %=",
            "^= &= |= && ||",
            "-> .* ##",
            "! ~ + - * / % =",            // 1
            "# ^ & | < > ( ) [ ] { } , . ? : ;",
            "\\" // TODO: replace in multi-line #define s?
        ].into_iter().flat_map(|l| l.split(' ')) {
            if src.starts_with(op) {
                let idx = self.cursor;
                self.cursor += op.len();
                //return Some(SrcToken { token: &src[..op.len()], idx });
                return Some(SrcToken { token: op, idx });
            }
        }

        fail!("unexpected character {:?}", src.chars().next().unwrap_or(' '));
    }

    pub fn idx_to_line_col_no(&self, idx: usize) -> (usize, usize) {
        let line_idx = self.eols.partition_point(|&eol_idx| eol_idx < idx);
        let col_idx = if line_idx == 0 { idx } else { idx - self.eols[line_idx-1] };
        (line_idx + 1, col_idx + 1)
    }

    pub fn idx_to_location(&self, idx: usize) -> Location {
        let (line_no, col_no) = self.idx_to_line_col_no(idx);
        Location {
            line_no:    NonZeroUsize::new(line_no),
            col_no:     NonZeroUsize::new(col_no),
            path:       self.path.clone(),
        }
    }

    pub fn token_to_line_col(&self, token: SrcToken) -> (usize, usize) {
        self.idx_to_line_col_no(token.idx)
    }

    pub fn token_to_location(&self, token: SrcToken) -> Location {
        self.idx_to_location(token.idx)
    }
}

#[test] fn test_idx_to_location() {
    let src = SrcReader::new(Path::new("a.txt").into(), "foo\nbar\nbaz");
    let line_no = "111 222 333";
    for (idx, ch) in line_no.char_indices() {
        match ch {
            '0' ..= '9' => {
                let line_no = ch as usize - '0' as usize;
                assert_eq!(src.idx_to_line_col_no(idx).0, line_no);
            },
            ' ' => continue,
            _other => panic!("unexpected line_no char {:?}", _other),
        }
    }
}

#[test] fn test_str_methods() {
    let src = SrcReader::new(Path::new("a.txt").into(), "foo\nbar\nbaz");
    let _ = src.starts_with("FOO");
    let _ = src.split_once_trim("FOO");
}
