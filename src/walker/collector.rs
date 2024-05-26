use super::{Entry, Tolerance, E};
use crate::breaker::Breaker;
use log::{debug, error, warn};
use std::{
    fs::{read_dir, read_link},
    mem,
    path::PathBuf,
    time::{Duration, Instant},
};

pub struct Collector<'a> {
    pub(crate) invalid: Vec<PathBuf>,
    pub(crate) collected: Vec<PathBuf>,
    pub(crate) entries: Vec<Entry>,
    breaker: &'a Breaker,
    tolerance: Tolerance,
}

impl<'a> Collector<'a> {
    pub fn new(tolerance: Tolerance, breaker: &'a Breaker, entries: Vec<Entry>) -> Self {
        Self {
            breaker,
            tolerance,
            invalid: Vec::new(),
            collected: Vec::new(),
            entries,
        }
    }

    fn report<Er: Into<E>, S: AsRef<str>>(
        &mut self,
        blamed: &PathBuf,
        log: S,
        err: Er,
    ) -> Result<(), E> {
        match self.tolerance {
            Tolerance::StopOnErrors => {
                error!("{}", log.as_ref());
                return Err(err.into());
            }
            Tolerance::LogErrors => {
                warn!("{}", log.as_ref());
                self.invalid.push(blamed.to_owned());
            }
            Tolerance::DoNotLogErrors => {
                self.invalid.push(blamed.to_owned());
            }
        };
        Ok(())
    }

    fn collect_from_symlink(&mut self, link: &PathBuf, entry: &Entry) -> Result<(), E> {
        let path = match read_link(&link) {
            Ok(path) => path,
            Err(err) => {
                return self.report(
                    &link,
                    format!(
                        "Fail to read symlink from {} due error: {err}",
                        link.to_string_lossy()
                    ),
                    (link.to_owned(), err),
                );
            }
        };
        self.collect_from_path(&path, entry)
    }

    fn collect_from_path(&mut self, path: &PathBuf, entry: &Entry) -> Result<(), E> {
        if self.breaker.is_aborded() {
            return Err(E::Aborted);
        }
        if path.is_file() {
            if entry.filtered(path) {
                self.collected.push(path.to_owned());
            }
            return Ok(());
        } else if path.is_symlink() {
            return self.collect_from_symlink(path, entry);
        }
        if !path.is_dir() || !entry.filtered(&path) {
            return Ok(());
        }
        let elements = match read_dir(&path) {
            Ok(elements) => elements,
            Err(err) => {
                return self.report(
                    &path,
                    format!(
                        "Fail to read directory {} due error: {err}",
                        path.to_string_lossy()
                    ),
                    (path.to_owned(), err),
                );
            }
        };
        for el in elements {
            match el {
                Ok(el) => {
                    self.collect_from_path(&el.path(), entry)?;
                }
                Err(err) => {
                    self.report(
                        &path,
                        format!(
                            "Fail to read entity from directory {} due error: {err}",
                            path.to_string_lossy()
                        ),
                        (path.to_owned(), err),
                    )?;
                }
            }
        }
        Ok(())
    }

    pub fn collect(&mut self) -> Result<(), E> {
        let now = Instant::now();
        let entries = mem::take(&mut self.entries);
        for entry in entries {
            self.collect_from_path(&entry.entry, &entry)?;
        }
        debug!(
            "collected {} paths in {}µs / {}ms / {}s",
            self.collected.len(),
            now.elapsed().as_micros(),
            now.elapsed().as_millis(),
            now.elapsed().as_secs()
        );
        Ok(())
    }
}
