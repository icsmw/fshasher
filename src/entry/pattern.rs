pub use super::E;
use core::panic;
use glob::Pattern;
use std::path::Path;

const MAX_DEPTH: usize = 1;

#[derive(Debug)]
pub enum PatternFilter<T: AsRef<str>> {
    Ignore(T),
    Accept(T),
    Cmb(Vec<PatternFilter<T>>),
}

impl<T: AsRef<str>> TryInto<PatternFilterAccepted> for PatternFilter<T> {
    type Error = E;

    fn try_into(self) -> Result<PatternFilterAccepted, Self::Error> {
        Ok(match self {
            Self::Ignore(s) => PatternFilterAccepted::Ignore(
                Pattern::new(s.as_ref())
                    .map_err(|err: glob::PatternError| (s.as_ref().to_string(), err))?,
            ),
            Self::Accept(s) => PatternFilterAccepted::Accept(
                Pattern::new(s.as_ref())
                    .map_err(|err: glob::PatternError| (s.as_ref().to_string(), err))?,
            ),
            Self::Cmb(filters) => {
                let mut patterns: Vec<PatternFilterAccepted> = Vec::new();
                for filter in filters {
                    patterns.push(filter.try_into()?);
                }
                PatternFilterAccepted::Cmb(patterns)
            }
        })
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum PatternFilterAccepted {
    Ignore(Pattern),
    Accept(Pattern),
    Cmb(Vec<PatternFilterAccepted>),
}

impl PatternFilterAccepted {
    pub fn filtered<P: AsRef<Path>>(&self, path: P) -> bool {
        self.filtered_with_depth(path.as_ref(), 0)
    }

    fn filtered_with_depth(&self, path: &Path, depth: usize) -> bool {
        if depth > MAX_DEPTH {
            panic!(
                "PatternFilterAccepted::Cmb cannot have nested PatternFilterAccepted::Cmb elements"
            );
        }
        match self {
            Self::Ignore(p) => !p.matches_path(path),
            Self::Accept(p) => p.matches_path(path),
            Self::Cmb(patterns) => !patterns
                .iter()
                .any(|p| !p.filtered_with_depth(path, depth + 1)),
        }
    }
}
