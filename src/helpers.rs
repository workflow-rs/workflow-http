use std::{fs, io::Result};
use std::path::Path;

pub fn copy_dir(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

type FilterCallback =  fn(bool, &str)->bool;
pub fn copy_directory(root:&impl AsRef<Path>, src: impl AsRef<Path>, dst: impl AsRef<Path>, filter:FilterCallback) -> Result<()> {
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(&src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let is_dir = ty.is_dir();
        let file_name = entry.file_name();
        if !filter(is_dir, &src.as_ref().join(&file_name).to_str().unwrap().replacen(root.as_ref().to_str().unwrap(), "", 1)){
            continue;
        }
        if is_dir {
            copy_directory(root, entry.path(), dst.as_ref().join(file_name), filter)?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(file_name))?;
        }
    }
    Ok(())
}


pub fn copy_dir_with_filter(src: impl AsRef<Path>, dst: impl AsRef<Path>, filter:FilterCallback) -> Result<()> {
    copy_directory(&src, &src, &dst, filter)
}