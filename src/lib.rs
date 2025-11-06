use std::collections::HashSet;
use std::fs;
use std::io::{self, Read, Write};
use std::os::unix::fs::{self as unix_fs, PermissionsExt};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct CopyOptions {
    pub overwrite: bool,
    pub follow_symlinks: bool,
    pub create_dest: bool,
    pub buffer_size: usize,
    pub max_depth: usize,
}

impl Default for CopyOptions {
    fn default() -> Self {
        Self {
            overwrite: true,
            follow_symlinks: false,
            create_dest: true,
            buffer_size: 8 * 1024,
            max_depth: 512,
        }
    }
}

#[derive(Debug)]
pub enum CopyError {
    Io(io::Error),
    DepthExceeded(PathBuf),
    SymlinkLoop(PathBuf),
    SrcNotFound(PathBuf),
    DestNotDir(PathBuf),
    NotSupported(PathBuf),
}

impl From<io::Error> for CopyError {
    fn from(e: io::Error) -> Self {
        CopyError::Io(e)
    }
}

pub fn copy_recursive(src: &Path, dst: &Path, opts: &CopyOptions) -> Result<(), CopyError> {
    if !src.exists() {
        return Err(CopyError::SrcNotFound(src.to_path_buf()));
    }

    if src.is_file() {
        let dest_path = if dst.is_dir() {
            dst.join(src.file_name().unwrap())
        } else {
            dst.to_path_buf()
        };
        copy_one(src, &dest_path, opts)?;
        return Ok(());
    }

    if src.is_dir() {
        if dst.exists() && !dst.is_dir() {
            return Err(CopyError::DestNotDir(dst.to_path_buf()));
        }
        if opts.create_dest {
            fs::create_dir_all(dst)?;
        }

        let mut visited = HashSet::new();
        copy_dir(src, dst, opts, 0, &mut visited)?;
        return Ok(());
    }

    Err(CopyError::NotSupported(src.to_path_buf()))
}

fn copy_dir(
    src: &Path,
    dst: &Path,
    opts: &CopyOptions,
    depth: usize,
    visited: &mut HashSet<PathBuf>,
) -> Result<(), CopyError> {
    if depth > opts.max_depth {
        return Err(CopyError::DepthExceeded(src.to_path_buf()));
    }

    let real_src = fs::canonicalize(src)?;
    if !visited.insert(real_src.clone()) {
        return Err(CopyError::SymlinkLoop(real_src));
    }

    for entry_res in fs::read_dir(src)? {
        let entry = entry_res?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        let meta = entry.metadata()?;

        if meta.is_dir() {
            if !dst_path.exists() {
                fs::create_dir_all(&dst_path)?;
            }
            copy_dir(&src_path, &dst_path, opts, depth + 1, visited)?;
        } else if meta.is_file() {
            copy_one(&src_path, &dst_path, opts)?;
        } else if meta.file_type().is_symlink() {
            handle_symlink(&src_path, &dst_path, opts, depth, visited)?;
        } else {
            eprintln!("Ignoring special file type: {}", src_path.display());
        }
    }
    Ok(())
}

fn handle_symlink(
    src: &Path,
    dst: &Path,
    opts: &CopyOptions,
    depth: usize,
    visited: &mut HashSet<PathBuf>,
) -> Result<(), CopyError> {
    if opts.follow_symlinks {
        let target = fs::canonicalize(src)?;
        if target.is_file() {
            copy_one(&target, dst, opts)?;
        } else if target.is_dir() {
            copy_dir(&target, dst, opts, depth + 1, visited)?;
        }
    } else {
        recreate_symlink(src, dst, opts)?;
    }
    Ok(())
}

fn copy_one(src: &Path, dst: &Path, opts: &CopyOptions) -> Result<(), CopyError> {
    if dst.exists() {
        if opts.overwrite {
            fs::remove_file(dst)?;
        } else {
            return Ok(());
        }
    } else if let Some(p) = dst.parent() {
        if opts.create_dest {
            fs::create_dir_all(p)?;
        }
    }

    let mut input = fs::File::open(src)?;
    let mut output = fs::File::create(dst)?;
    let mut buf = vec![0u8; opts.buffer_size];

    loop {
        let n = input.read(&mut buf)?;
        if n == 0 {
            break;
        }
        output.write_all(&buf[..n])?;
    }

    let mode = fs::metadata(src)?.permissions().mode();
    let mut perms = fs::metadata(dst)?.permissions();
    perms.set_mode(mode);
    fs::set_permissions(dst, perms)?;

    Ok(())
}

fn recreate_symlink(src: &Path, dst: &Path, opts: &CopyOptions) -> Result<(), CopyError> {
    let target = fs::read_link(src)?;
    if dst.exists() {
        if opts.overwrite {
            fs::remove_file(dst)?;
        } else {
            return Ok(());
        }
    }
    unix_fs::symlink(&target, dst)?;
    Ok(())
}

#[cfg(test)]
mod tests;
