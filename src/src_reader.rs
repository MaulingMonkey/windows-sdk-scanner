use crate::*;

use std::num::NonZeroUsize;
use std::path::*;
use std::sync::*;



pub(crate) struct SrcReader<'t> {
    path:               Arc<Path>,
    //full_source:        &'t str,
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
            //full_source:        source,
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
}

fn one() -> NonZeroUsize { // TODO: const fn when stable
    NonZeroUsize::new(1).unwrap()
}

fn inc(nz: &mut NonZeroUsize) {
    *nz = NonZeroUsize::new(nz.get() + 1).unwrap_or(*nz);
}
