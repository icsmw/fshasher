use crate::{collector::E, entry::ContextFileAccepted};
use glob::Pattern;
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufRead},
    path::{Path, PathBuf},
};
#[derive(Default, Debug)]
struct ContextPatterns {
    accept: Vec<(Pattern, bool)>,
    ignore: Vec<(Pattern, bool)>,
}

impl ContextPatterns {
    pub fn append(
        &mut self,
        context_file: &ContextFileAccepted,
        mut patterns: Vec<(Pattern, bool)>,
    ) {
        if patterns.is_empty() {
            return;
        }
        match context_file {
            ContextFileAccepted::Accept(_) => self.accept.append(&mut patterns),
            ContextFileAccepted::Ignore(_) => self.ignore.append(&mut patterns),
        }
    }

    pub fn merge(&mut self, pattern: &ContextPatterns) {
        for (pattern, ng) in pattern.ignore.iter() {
            if !self.ignore.iter().any(|(p, _)| p == pattern) {
                self.ignore.push((pattern.clone(), *ng))
            }
        }
    }

    pub fn filtered(&self, path: &Path) -> bool {
        if self.ignore.is_empty() && self.accept.is_empty() {
            return true;
        }
        // Path is exeption in ingore; no accept filter
        if self
            .ignore
            .iter()
            .filter(|(_, negative)| *negative)
            .any(|(p, _)| p.matches_path(path))
            && self.accept.is_empty()
        {
            return true;
        }
        // Path is ignored
        if self
            .ignore
            .iter()
            .filter(|(_, negative)| !*negative)
            .any(|(p, _)| p.matches_path(path))
        {
            return false;
        }
        // Path isn't ignored; no accept lists
        if self.accept.is_empty() {
            return true;
        }
        // Accept filtering is applying only to files
        if !path.is_file() {
            return true;
        }
        // Path is exeption in accept list
        if self
            .accept
            .iter()
            .filter(|(_, negative)| *negative)
            .any(|(p, _)| p.matches_path(path))
        {
            return false;
        }
        // Path is in accept list or not
        self.accept
            .iter()
            .filter(|(_, negative)| !*negative)
            .any(|(p, _)| p.matches_path(path))
    }
}

#[derive(Debug)]
pub struct Context {
    files: Vec<ContextFileAccepted>,
    patterns: HashMap<PathBuf, ContextPatterns>,
}

impl Context {
    pub fn new(files: &[ContextFileAccepted]) -> Self {
        Self {
            files: files.to_vec(),
            patterns: HashMap::new(),
        }
    }
    pub fn consider(&mut self, parent: &PathBuf) -> Result<(), E> {
        let mut context_patterns = ContextPatterns::default();
        for file in self.files.iter() {
            if let Some(filepath) = file.filepath(parent) {
                context_patterns.append(file, self.parse(&filepath)?);
            }
        }
        if !context_patterns.ignore.is_empty() || !context_patterns.accept.is_empty() {
            for rel in self.relevant_to(parent) {
                context_patterns.merge(rel);
            }
            self.patterns.insert(parent.clone(), context_patterns);
        }
        Ok(())
    }

    pub fn filtered(&self, path: &Path) -> bool {
        if let Some(cx) = self.relevant_to(path).first() {
            cx.filtered(path)
        } else {
            true
        }
    }

    fn relevant_to(&self, path: &Path) -> Vec<&ContextPatterns> {
        let mut current = path;
        let mut all = Vec::new();
        while let Some(parent) = current.parent() {
            if let Some(patterns) = self.patterns.get(&parent.to_path_buf()) {
                all.push(patterns)
            }
            current = parent;
        }
        all
    }

    fn parse(&self, filename: &PathBuf) -> Result<Vec<(Pattern, bool)>, E> {
        let file = File::open(filename)?;
        let reader = io::BufReader::new(file);
        let mut patterns: Vec<(Pattern, bool)> = Vec::new();
        for ln in reader.lines().collect::<io::Result<Vec<String>>>()? {
            if ln.trim().is_empty() {
                continue;
            }
            if ln.trim().starts_with('#') {
                continue;
            }
            let negative = ln.trim().starts_with('!');
            let pattern = Pattern::new(if negative {
                &ln.trim()[1..]
            } else {
                &ln.trim()[0..]
            })
            .map_err(|err: glob::PatternError| (ln.to_string(), err))?;
            patterns.push((pattern, negative))
        }
        Ok(patterns)
    }
}
