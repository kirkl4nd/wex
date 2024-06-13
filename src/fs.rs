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

    /// Parses and sanitizes the input path, ensuring it's within the base directory.
    pub fn parse_path(&self, request_path: &str) -> Option<PathBuf> {
        let path = self.base_path.join(request_path).canonicalize().ok()?;
        if path.starts_with(&self.base_path) {
            Some(path)
        } else {
            None
        }
    }

    /// Determines the type of the file system entry (file, directory, or none).
    pub fn path_type(&self, path: &Path) -> Option<fs::FileType> {
        fs::metadata(path).ok().map(|meta| meta.file_type())
    }

    /// Lists the contents of a directory.
    pub fn list_directory(&self, dir_path: &Path) -> io::Result<Vec<PathBuf>> {
        fs::read_dir(dir_path)?
            .map(|res| res.map(|e| e.path()))
            .collect()
    }

    /// Creates a new file at the specified path.
    pub fn create_file(&self, file_path: &Path) -> io::Result<()> {
        File::create(file_path).map(|_| ())
    }

    /// Creates a new directory at the specified path.
    pub fn create_directory(&self, dir_path: &Path) -> io::Result<()> {
        fs::create_dir_all(dir_path)
    }

    /// Deletes a file at the specified path.
    pub fn delete_file(&self, file_path: &Path) -> io::Result<()> {
        fs::remove_file(file_path)
    }

    /// Deletes a directory at the specified path.
    pub fn delete_directory(&self, dir_path: &Path) -> io::Result<()> {
        fs::remove_dir_all(dir_path)
    }

    /// Reads and returns the contents of a file.
    pub fn read_file_contents(&self, file_path: &Path) -> io::Result<Vec<u8>> {
        fs::read(file_path)
    }
}
