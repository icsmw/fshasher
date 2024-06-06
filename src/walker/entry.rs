use super::E;
use glob::Pattern;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum Filter<T: AsRef<str>> {
    Folders(T),
    Files(T),
    Common(T),
}

impl<T: AsRef<str>> Filter<T> {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Files(s) => s.as_ref(),
            Self::Folders(s) => s.as_ref(),
            Self::Common(s) => s.as_ref(),
        }
    }
}

impl<T: AsRef<str>> TryInto<FilterAccepted> for Filter<T> {
    type Error = E;

    fn try_into(self) -> Result<FilterAccepted, Self::Error> {
        let pattern =
            Pattern::new(self.as_str()).map_err(|err| (self.as_str().to_string(), err))?;
        Ok(match self {
            Self::Files(..) => FilterAccepted::Files(pattern),
            Self::Folders(..) => FilterAccepted::Folders(pattern),
            Self::Common(..) => FilterAccepted::Common(pattern),
        })
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone)]
pub enum FilterAccepted {
    Folders(Pattern),
    Files(Pattern),
    Common(Pattern),
}

impl FilterAccepted {
    pub fn filtered<P: AsRef<Path>>(&self, full_path: P) -> Option<bool> {
        let path = match self {
            Self::Files(..) => Path::new(full_path.as_ref().file_name()?),
            Self::Folders(..) => full_path.as_ref(),
            Self::Common(..) => full_path.as_ref(),
        };
        Some(
            match self {
                Self::Files(p) => {
                    if full_path.as_ref().is_file() {
                        p
                    } else {
                        return None;
                    }
                }
                Self::Folders(p) => {
                    if full_path.as_ref().is_dir() {
                        p
                    } else {
                        return None;
                    }
                }
                Self::Common(p) => p,
            }
            .matches_path(path),
        )
    }
}

#[derive(Default, Debug, Clone)]
pub struct Entry {
    pub entry: PathBuf,
    pub include: Vec<FilterAccepted>,
    pub exclude: Vec<FilterAccepted>,
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

    pub fn filtered<P: AsRef<Path>>(&self, path: P) -> bool {
        if self.exclude.iter().any(|filter| {
            if let Some(v) = filter.filtered(&path) {
                v
            } else {
                false
            }
        }) {
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
