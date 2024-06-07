use super::{Entry, Filter, Progress, ProgressChannel, Walker, E};
use crate::{Hasher, Reader};
use std::{mem, ops::Range, path::Path};

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

#[derive(Debug, Clone, Default)]
pub enum ReadingStrategy {
    #[default]
    Buffer,
    Complete,
    MemoryMapped,
    Scenario(Vec<(Range<u64>, Box<ReadingStrategy>)>),
}

#[derive(Default, Debug)]
pub struct Options {
    pub(crate) tolerance: Tolerance,
    pub(crate) entries: Vec<Entry>,
    pub(crate) global: Entry,
    pub(crate) progress: Option<ProgressChannel>,
    pub(crate) threads: Option<usize>,
    pub(crate) reading_strategy: ReadingStrategy,
}

impl Options {
    pub fn new() -> Self {
        Self {
            tolerance: Tolerance::LogErrors,
            entries: Vec::new(),
            global: Entry::default(),
            progress: None,
            threads: None,
            reading_strategy: ReadingStrategy::default(),
        }
    }

    pub fn from<P: AsRef<Path>>(path: P) -> Result<Self, E> {
        Ok(Self {
            tolerance: Tolerance::LogErrors,
            entries: vec![Entry::from(path)?],
            global: Entry::default(),
            progress: None,
            threads: None,
            reading_strategy: ReadingStrategy::default(),
        })
    }

    pub fn reading_strategy(&mut self, reading_strategy: ReadingStrategy) -> Result<&mut Self, E> {
        if let ReadingStrategy::Scenario(scenario) = &reading_strategy {
            let mut from = 0;
            for (range, strategy) in scenario.iter() {
                if matches!(**strategy, ReadingStrategy::Scenario(_)) {
                    return Err(E::NestedtedScenarioStrategy);
                }
                if range.start != from {
                    return Err(E::InvalidRangesForScenarioStrategy(from));
                }
                from = range.end;
            }
        }
        self.reading_strategy = reading_strategy;
        Ok(self)
    }

    pub fn threads(&mut self, threads: usize) -> &mut Self {
        self.threads = Some(threads);
        self
    }

    pub fn progress(&mut self, capacity: usize) -> &mut Self {
        self.progress = Some(Progress::channel(capacity));
        self
    }

    pub fn tolerance(&mut self, tolerance: Tolerance) -> &mut Self {
        self.tolerance = tolerance;
        self
    }

    pub fn path<P: AsRef<Path>>(&mut self, path: P) -> Result<&mut Self, E> {
        self.entries.push(Entry::from(path)?);
        Ok(self)
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

    pub fn walker<H: Hasher + 'static, R: Reader + 'static>(
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
                threads: self.threads.take(),
                reading_strategy: self.reading_strategy.clone(),
            },
            hasher,
            reader,
        )
    }
}
