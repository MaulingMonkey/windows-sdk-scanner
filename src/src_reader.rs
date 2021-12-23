use crate::*;

use std::num::NonZeroUsize;
use std::ops::Deref;
use std::path::*;
use std::sync::*;



pub(crate) struct SrcReader<'t> {
    path:               Arc<Path>,
    eols:               Vec<usize>,
    full_source:        &'t str,
    remaining_source:   &'t str,
    next_line_no:       NonZeroUsize,
    next_column_no:     NonZeroUsize,
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
            remaining_source:   source,
            next_line_no:       one(),
            next_column_no:     one(),
        }
    }

    pub fn next_line(&mut self) -> Option<SrcLine<'t>> {
        if self.remaining_source.is_empty() { return None; }
        let eol = self.remaining_source.find('\n').unwrap_or(self.remaining_source.len());

        let raw         = &self.remaining_source[..eol].trim_end_matches('\r');
        let trimmed     = raw.trim();
        let path        = self.path.clone();
        let line_no     = Some(self.next_line_no);
        let col_no      = Some(self.next_column_no);
        let location    = Location { path, line_no, col_no };

        inc(&mut self.next_line_no);
        self.next_column_no     = one();
        self.remaining_source   = self.remaining_source.get(eol+1..).unwrap_or("");

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

fn one() -> NonZeroUsize { // TODO: const fn when stable
    NonZeroUsize::new(1).unwrap()
}

fn inc(nz: &mut NonZeroUsize) {
    *nz = NonZeroUsize::new(nz.get() + 1).unwrap_or(*nz);
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
