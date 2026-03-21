use std::path::Path;

pub trait IsabellePathContext {
    fn to_isabelle_path(&self) -> Option<String>;
}

impl IsabellePathContext for Path {
    fn to_isabelle_path(&self) -> Option<String> {
        let full_path = dunce::canonicalize(self).ok()?;
        let path_str = full_path.to_str()?;

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
