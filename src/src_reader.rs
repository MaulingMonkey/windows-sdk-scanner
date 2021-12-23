use crate::*;

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

impl<'t> SrcReader<'t> {
    pub fn new(path: Arc<Path>, source: &'t str) -> Self {
        Self {
            path,
            eols:               source.char_indices().filter(|(_, ch)| *ch == '\n').map(|(i, _)| i).chain(Some(source.len())).collect(),
            full_source:        source,
            cursor:             0,
        }
    }

    pub fn next_line(&mut self) -> Option<SrcLine<'t>> {
        let remaining_source = self.full_source.get(self.cursor..).unwrap_or("");
        if remaining_source.is_empty() { return None; }
        let eol = remaining_source.find('\n').unwrap_or(remaining_source.len());

        let raw         = remaining_source[..eol].trim_end_matches('\r');
        let trimmed     = raw.trim();
        let location    = self.idx2loc(self.cursor);
        self.cursor     += eol + 1;

        Some( SrcLine { location, raw, trimmed } )
    }

    pub fn idx2loc(&self, idx: usize) -> Location {
        let line_idx = self.eols.partition_point(|&eol_idx| eol_idx < idx);
        let col_idx = if line_idx == 0 { idx } else { idx - self.eols[line_idx-1] };
        Location {
            line_no:    NonZeroUsize::new(line_idx + 1),
            col_no:     NonZeroUsize::new(col_idx + 1),
            path:       self.path.clone(),
        }
    }
}

impl Deref for SrcReader<'_> {
    type Target = str;
    fn deref(&self) -> &Self::Target { self.full_source }
}

#[test] fn test_idx2loc() {
    let src = SrcReader::new(Path::new("a.txt").into(), "foo\nbar\nbaz");
    let line_no = "111 222 333";
    for (idx, ch) in line_no.char_indices() {
        match ch {
            '0' ..= '9' => {
                let line_no = ch as usize - '0' as usize;
                assert_eq!(src.idx2loc(idx).line_no_or_0(), line_no);
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
