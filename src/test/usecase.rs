use log::debug;
use rand::Rng;
use std::{
    env::temp_dir,
    fs::{create_dir, remove_dir_all, OpenOptions},
    io::{self, Write},
    path::PathBuf,
    time::Instant,
};
use uuid::Uuid;

pub struct UseCase {
    pub files: Vec<PathBuf>,
    pub root: PathBuf,
}

impl UseCase {
    pub fn gen(
        folders_number: u16,
        deep: u8,
        files_per_folder: usize,
        exts: &[&str],
    ) -> Result<Self, io::Error> {
        let now = Instant::now();
        debug!("Start generiting use case: {folders_number} folders; deep = {deep}; {files_per_folder} files per folder; exts: {}", exts.join(", "));
        let mut files = Vec::new();
        let mut fill = |parent: &PathBuf| -> Result<Vec<PathBuf>, io::Error> {
            let mut created = Vec::new();
            for _ in 0..folders_number {
                let folder = parent.join(Uuid::new_v4().to_string());
                create_dir(&folder)?;
                let mut ext = 0;
                for _ in 0..files_per_folder {
                    if ext >= exts.len() {
                        ext = 0;
                    }
                    let mut filename = folder.join(Uuid::new_v4().to_string());
                    exts.get(ext).map(|ext| filename.set_extension(ext));
                    ext += 1;
                    let mut file = OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(&filename)?;
                    file.write_all(Uuid::new_v4().as_bytes())?;
                    file.flush()?;
                    files.push(filename);
                }
                created.push(folder);
            }
            Ok(created)
        };
        let tmp = temp_dir();
        let root = tmp.join(Uuid::new_v4().to_string());
        if root.exists() {
            remove_dir_all(&root)?;
        }
        create_dir(&root)?;
        let mut folders: Vec<PathBuf> = fill(&root)?;
        for _ in 0..deep {
            let to_be_processed: Vec<PathBuf> = folders.to_vec();
            folders = Vec::new();
            for folder in to_be_processed.iter() {
                folders.append(&mut fill(folder)?);
            }
        }
        debug!(
            "in \"{}\" created {} files in {}µs / {}ms / {}s",
            root.display(),
            files.len(),
            now.elapsed().as_micros(),
            now.elapsed().as_millis(),
            now.elapsed().as_secs()
        );
        Ok(Self { files, root })
    }

    pub fn change(&self) -> Result<(), io::Error> {
        if self.files.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "No files has been created. Cannot change a state",
            ));
        }
        let Some(filename) = self
            .files
            .get(rand::thread_rng().gen_range(0..self.files.len() - 1))
        else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Cannot find a file path by index",
            ));
        };
        let mut file = OpenOptions::new().append(true).open(filename)?;
        file.write_all(Uuid::new_v4().as_bytes())?;
        file.flush()?;
        Ok(())
    }

    pub fn clean(&self) -> Result<(), io::Error> {
        if !self.root.exists() {
            return Ok(());
        }
        let Some(parent) = self.root.parent() else {
            return Ok(());
        };
        if !parent.starts_with(temp_dir()) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("parent of root isn't belong to {}", temp_dir().display()),
            ));
        }
        remove_dir_all(&self.root)?;
        debug!("Removed {}", self.root.display());
        Ok(())
    }
}
