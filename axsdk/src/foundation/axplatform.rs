use std::path::Path;
use std::fs;

// TODO(mdeforge): TBH, I'm not sure we need this API anymore with how good std::fs is...

pub fn file_exists(path: &Path) -> bool {
    path.exists()
}

pub fn directory_exists(path: &Path) -> bool {
    path.is_dir()
}

pub fn create_directory(path: &Path) -> std::io::Result<()> {
    fs::create_dir_all(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_fs::{prelude::*, TempDir};

    #[test]
    fn test_file_exists() {
        let temp_dir = TempDir::new().unwrap();
        let input_file = temp_dir.child("test.txt");
        input_file.touch().unwrap();

        assert!(file_exists(input_file.path()));
        assert!(!file_exists(Path::new("foo.txt")));

        temp_dir.close().unwrap();
    }

    #[test]
    fn test_directory_exists() {
        let temp_dir = TempDir::new().unwrap();
        let missing_dir = Path::new(temp_dir.path()).join("asdf");
        let new_dir = temp_dir.child("subdir");
        new_dir.create_dir_all().unwrap();

        assert!(directory_exists(new_dir.path()));
        assert!(!directory_exists(missing_dir.as_path()));

        temp_dir.close().unwrap();
    }

    #[test]
    fn test_create_directory() {
        let temp_dir = TempDir::new().unwrap();
        let new_dir = temp_dir.child("test");

        create_directory(new_dir.path()).unwrap();
        assert!(directory_exists(new_dir.path()));

        temp_dir.close().unwrap();
    }
}