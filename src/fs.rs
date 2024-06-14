use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

pub struct FileManager {
    base_path: PathBuf,
}

impl FileManager {
    /// Creates a new FileManager instance with a specified base directory.
    pub fn new(base_path: PathBuf) -> Self {
        FileManager { base_path }
    }

    /// Helper method to resolve a relative path to an absolute path within the base directory.
    fn resolve_path(&self, rel_path: &str) -> io::Result<PathBuf> {
        let sanitized_path = Path::new(rel_path).strip_prefix("../").unwrap_or(Path::new(rel_path));
        let full_path = self.base_path.join(sanitized_path);
        let canonical_path = full_path.canonicalize()?;
        if canonical_path.starts_with(&self.base_path) {
            Ok(canonical_path)
        } else {
            Err(io::Error::new(io::ErrorKind::PermissionDenied, "Path is outside the base directory"))
        }
    }

    /// Lists the contents of a directory given a relative path.
    pub fn list_directory(&self, rel_dir_path: &str) -> io::Result<Vec<PathBuf>> {
        let dir_path = self.resolve_path(rel_dir_path)?;
        fs::read_dir(dir_path)?
            .map(|res| res.map(|e| e.path()))
            .collect()
    }

    /// Creates a new file at the specified relative path.
    pub fn create_file(&self, rel_file_path: &str) -> io::Result<()> {
        let file_path = self.resolve_path(rel_file_path)?;
        File::create(file_path).map(|_| ())
    }

    /// Creates a new directory at the specified relative path.
    pub fn create_directory(&self, rel_dir_path: &str) -> io::Result<()> {
        let dir_path = self.resolve_path(rel_dir_path)?;
        fs::create_dir_all(dir_path)
    }

    /// Deletes a file at the specified relative path.
    pub fn delete_file(&self, rel_file_path: &str) -> io::Result<()> {
        let file_path = self.resolve_path(rel_file_path)?;
        fs::remove_file(file_path)
    }

    /// Deletes a directory at the specified relative path.
    pub fn delete_directory(&self, rel_dir_path: &str) -> io::Result<()> {
        let dir_path = self.resolve_path(rel_dir_path)?;
        fs::remove_dir_all(dir_path)
    }

    /// Reads and returns the contents of a file given a relative path.
    pub fn read_file_contents(&self, rel_file_path: &str) -> io::Result<Vec<u8>> {
        let file_path = self.resolve_path(rel_file_path)?;
        fs::read(file_path)
    }
}
