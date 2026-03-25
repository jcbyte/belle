use std::path::Path;

pub trait IsabellePathContext {
    fn to_isabelle_path(&self) -> Option<String>;
}

impl IsabellePathContext for Path {
    fn to_isabelle_path(&self) -> Option<String> {
        // Ensure an absolute path is passed
        if !self.is_absolute() {
            return None;
        }

        let path_str = self.to_str()?;

        if cfg!(windows) {
            // On windows we must convert to a cygpath
            let path_str = path_str.replace("\\", "/");
            let (drive, path) = path_str.split_once(":/")?;
            return Some(format!("/cygdrive/{}/{}", drive.to_ascii_lowercase(), path));
        }

        // On linux, this is expected by isabelle
        Some(path_str.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_rejects_relative_path() {
        let path = Path::new("relative/path/to/file");
        assert_eq!(path.to_isabelle_path(), None);
    }

    #[cfg(not(windows))]
    #[test]
    fn test_linux_absolute_path() {
        let path = Path::new("/home/user/isabelle");
        let isabelle_path = path.to_isabelle_path();
        assert!(isabelle_path.is_some());
        assert_eq!(isabelle_path.unwrap(), "/home/user/isabelle");
    }

    #[cfg(windows)]
    #[test]
    fn test_windows_cygpath_conversion() {
        let path = Path::new(r"C:\Users\Guest\Project");
        let isabelle_path = path.to_isabelle_path();
        assert!(isabelle_path.is_some());
        assert_eq!(isabelle_path.unwrap(), "/cygdrive/c/Users/Guest/Project");

        let path = Path::new(r"D:\Data");
        let isabelle_path = path.to_isabelle_path();
        assert!(isabelle_path.is_some());
        assert_eq!(isabelle_path.unwrap(), "/cygdrive/d/Data");
    }
}
