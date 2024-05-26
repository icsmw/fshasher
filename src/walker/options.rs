use super::{Entry, Filter, Progress, Walker, E};
use crate::{Hasher, Reader};
use std::mem;

#[derive(Debug, Clone)]
pub enum Tolerance {
    LogErrors,
    DoNotLogErrors,
    StopOnErrors,
}

impl Default for Tolerance {
    fn default() -> Self {
        Self::LogErrors
    }
}

#[derive(Default, Debug)]
pub struct Options {
    pub(crate) tolerance: Tolerance,
    pub(crate) entries: Vec<Entry>,
    pub(crate) global: Entry,
    pub(crate) progress: Option<Progress>,
}

impl Options {
    pub fn new() -> Self {
        Self {
            tolerance: Tolerance::LogErrors,
            entries: Vec::new(),
            global: Entry::default(),
            progress: None,
        }
    }

    pub fn progress(&mut self) -> &mut Self {
        self.progress = Some(Progress::new());
        self
    }

    pub fn tolerance(&mut self, tolerance: Tolerance) -> &mut Self {
        self.tolerance = tolerance;
        self
    }

    pub fn entry(&mut self, entry: Entry) -> Result<&mut Self, E> {
        if !entry.entry.is_absolute() {
            return Err(E::RelativePathAsEntry(entry.entry));
        }
        self.entries.push(entry);
        Ok(self)
    }

    pub fn include<T: AsRef<str>>(&mut self, filter: Filter<T>) -> Result<&mut Self, E> {
        self.global.include(filter)?;
        Ok(self)
    }

    pub fn exclude<T: AsRef<str>>(&mut self, filter: Filter<T>) -> Result<&mut Self, E> {
        self.global.exclude(filter)?;
        Ok(self)
    }

    pub fn walker<H: Hasher, R: Reader>(
        &mut self,
        hasher: H,
        reader: R,
    ) -> Result<Walker<H, R>, E> {
        Walker::new(
            Options {
                tolerance: mem::take(&mut self.tolerance),
                global: mem::take(&mut self.global),
                entries: mem::take(&mut self.entries),
                progress: self.progress.take(),
            },
            hasher,
            reader,
        )
    }
}
