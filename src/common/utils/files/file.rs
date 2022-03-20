use std::fs::{create_dir_all, File};
use std::io;
use std::path::Path;

pub fn create_file(filename: &str) -> io::Result<()> {
    if exists(filename) {
        return Ok(());
    }

    let p = std::path::Path::new(filename);
    let parent_path = p.parent();
    match parent_path {
        None => (),
        Some(d) => {
            let path_str = path_to_str(d);
            match path_str {
                None => (),
                Some(d) => create_directory(d)?,
            };
        }
    };

    let f = File::create(filename);

    match f {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn create_directory(dirname: &str) -> io::Result<()> {
    if exists(dirname) {
        return Ok(());
    }

    let f = create_dir_all(dirname);

    match f {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn exists(filename: &str) -> bool {
    Path::new(filename).exists()
}

pub fn path_to_str(path: &Path) -> Option<&str> {
    let f = path.file_name();

    return match f {
        None => None,
        Some(d) => d.to_str(),
    };
}
