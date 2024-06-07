mod error;
mod filter;
mod pattern;
pub use error::E;
pub use filter::Filter;
pub(crate) use filter::FilterAccepted;
pub use pattern::PatternFilter;
pub(crate) use pattern::PatternFilterAccepted;
use std::path::{Path, PathBuf};

#[derive(Default, Debug, Clone)]
pub struct Entry {
    pub entry: PathBuf,
    pub include: Vec<FilterAccepted>,
    pub exclude: Vec<FilterAccepted>,
    pub patterns: Vec<PatternFilterAccepted>,
}

impl Entry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from<T: AsRef<Path>>(path: T) -> Result<Self, E> {
        let mut entry = Self::new();
        entry.entry(path)?;
        Ok(entry)
    }

    pub fn entry<T: AsRef<Path>>(&mut self, path: T) -> Result<&mut Self, E> {
        let path = path.as_ref().to_path_buf();
        if !path.is_absolute() {
            return Err(E::RelativePathAsEntry(path));
        } else if !path.is_dir() {
            return Err(E::OnlyFolderAsCwd(path));
        }
        self.entry = path;
        Ok(self)
    }

    pub fn include<T: AsRef<str>>(&mut self, filter: Filter<T>) -> Result<&mut Self, E> {
        let accepted: FilterAccepted = filter.try_into()?;
        if !self.include.contains(&accepted) && !self.exclude.contains(&accepted) {
            self.include.push(accepted);
        }
        Ok(self)
    }

    pub fn exclude<T: AsRef<str>>(&mut self, filter: Filter<T>) -> Result<&mut Self, E> {
        let accepted: FilterAccepted = filter.try_into()?;
        if !self.include.contains(&accepted) && !self.exclude.contains(&accepted) {
            self.exclude.push(accepted);
        }
        Ok(self)
    }

    pub fn pattern<T: AsRef<str>>(&mut self, pattern: PatternFilter<T>) -> Result<&mut Self, E> {
        let accepted: PatternFilterAccepted = pattern.try_into()?;
        if !self.patterns.contains(&accepted) {
            self.patterns.push(accepted);
        }
        Ok(self)
    }

    pub fn filtered<P: AsRef<Path>>(&self, path: P) -> bool {
        if !self.patterns.is_empty() {
            return self.patterns.iter().any(|pattern| pattern.filtered(&path));
        } else if self
            .exclude
            .iter()
            .any(|filter| filter.filtered(&path).unwrap_or_default())
        {
            return false;
        }
        if self.include.is_empty() {
            true
        } else {
            self.include.iter().any(|filter| {
                if let Some(v) = filter.filtered(&path) {
                    v
                } else {
                    true
                }
            })
        }
    }
}
