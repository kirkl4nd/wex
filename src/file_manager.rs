use std::fs::{self, File};
use std::io::{self};
use std::path::PathBuf;

pub struct FileManager {
    base_path: PathBuf,
}

impl FileManager {
    /// Creates a new FileManager instance with a specified base directory.
    pub fn new(base_path: PathBuf) -> Self {
        FileManager { base_path }
    }

    /// Helper function to construct a full path from a relative path, ensuring it's within the base path.
    fn full_path(&self, relative_path: &str) -> io::Result<PathBuf> {
        let mut path = self.base_path.clone();
        for component in PathBuf::from(relative_path).components() {
            match component {
                std::path::Component::ParentDir => {
                    // Only pop if it does not go beyond the base path
                    if path != self.base_path {
                        path.pop();
                    } else {
                        return Err(io::Error::new(io::ErrorKind::PermissionDenied, "Access denied"));
                    }
                },
                std::path::Component::Normal(part) => path.push(part),
                _ => (), // Skip root and current dir components
            }
        }

        // No need to check if it starts with base_path because we control the construction
        Ok(path)
    }

    /// Determines the type of the file system entry (file, directory, or none).
    pub fn path_type(&self, relative_path: &str) -> io::Result<fs::FileType> {
        let path = self.full_path(relative_path)?;
        fs::metadata(path).map(|meta| meta.file_type())
    }

    /// Lists the contents of a directory.
    pub fn list_directory(&self, relative_path: &str) -> io::Result<Vec<PathBuf>> {
        let dir_path = self.full_path(relative_path)?;
        fs::read_dir(dir_path)?
            .map(|res| res.map(|e| e.path()))
            .collect()
    }

    /// Creates a new directory at the specified path.
    pub fn create_directory(&self, relative_path: &str) -> io::Result<()> {
        let dir_path = self.full_path(relative_path)?;
        fs::create_dir_all(dir_path)
    }

    /// Deletes a file at the specified path.
    pub fn delete_file(&self, relative_path: &str) -> io::Result<()> {
        let file_path = self.full_path(relative_path)?;
        fs::remove_file(file_path)
    }

    /// Deletes a directory at the specified path.
    pub fn delete_directory(&self, relative_path: &str) -> io::Result<()> {
        let dir_path = self.full_path(relative_path)?;
        fs::remove_dir_all(dir_path)
    }

    /// Reads and returns the contents of a file.
    pub fn read_file_contents(&self, relative_path: &str) -> io::Result<Vec<u8>> {
        let file_path = self.full_path(relative_path)?;
        fs::read(file_path)
    }

    /// Writes the contents to the specified file path.
    pub fn write_file_contents(&self, rel_path: &str, contents: &[u8]) -> io::Result<()> {
        println!("{:?}", rel_path);
        let file_path = self.full_path(rel_path)?;
        fs::write(file_path, contents)
    }

    /// Moves or renames a file or directory from one path to another.
    pub fn move_file_or_directory(&self, from_relative_path: &str, to_relative_path: &str) -> io::Result<()> {
        let from_path = self.full_path(from_relative_path)?;
        let to_path = self.full_path(to_relative_path)?;

        fs::rename(from_path, to_path)
    }
}
