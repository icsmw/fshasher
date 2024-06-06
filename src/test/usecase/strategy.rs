use std::{
    fmt,
    path::{Path, PathBuf},
};
use uuid::Uuid;

#[derive(Clone)]
pub enum Strategy {
    Number(u16),
    Named(Vec<String>),
}

impl Strategy {
    pub fn count(&self) -> u16 {
        match self {
            Self::Named(l) => l.len() as u16,
            Self::Number(n) => *n,
        }
    }
    pub fn get_cursor(&self, parent: &Path) -> Cursor {
        Cursor {
            strategy: self.clone(),
            parent: parent.to_path_buf(),
            index: 0,
        }
    }
}

impl fmt::Display for Strategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Number(n) => format!("{n} folder per level"),
                Self::Named(l) => format!("{} named folder per level", l.len()),
            }
        )
    }
}

pub struct Cursor {
    strategy: Strategy,
    parent: PathBuf,
    index: usize,
}

impl Cursor {
    pub fn next(&mut self) -> PathBuf {
        match &self.strategy {
            Strategy::Number(_) => self.parent.join(Uuid::new_v4().to_string()),
            Strategy::Named(names) => {
                if self.index >= names.len() {
                    self.index = 0;
                }
                let index = self.index;
                self.index += 1;
                self.parent.join(
                    names
                        .get(index)
                        .expect("Strategy::Named should not be empty"),
                )
            }
        }
    }
}
