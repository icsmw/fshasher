use std::path::PathBuf;

pub struct Extentions {
    exts: Vec<String>,
    index: usize,
}

impl Extentions {
    pub fn from(exts: &[&str]) -> Self {
        Self {
            exts: exts.iter().map(|s| s.to_string()).collect(),
            index: 0,
        }
    }

    pub fn apply(&mut self, filename: &mut PathBuf) {
        if self.exts.is_empty() {
            return;
        }
        if self.index >= self.exts.len() {
            self.index = 0;
        }
        let index = self.index;
        self.index += 1;
        filename.set_extension(self.exts.get(index).expect("List of extention isn't empty"));
    }
}
