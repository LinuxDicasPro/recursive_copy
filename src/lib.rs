use std::collections::HashSet;
use std::fs;
use std::io::{self, copy};
use std::os::unix::fs::{self as unix_fs, PermissionsExt};
use std::path::{Path, PathBuf};
use walkdir_minimal::{WalkDir, WalkError};

#[derive(Clone, Debug)]
pub struct CopyOptions {
    pub overwrite: bool,
    pub follow_symlinks: bool,
    pub content_only: bool,
    pub buffer_size: usize,
    pub depth: usize,
}

impl Default for CopyOptions {
    fn default() -> Self {
        Self {
            overwrite: false,
            follow_symlinks: false,
            content_only: false,
            buffer_size: 64 * 1024,
            depth: 512,
        }
    }
}

#[derive(Debug)]
pub enum CopyError {
    Io(io::Error),
    Walk(WalkError),
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
            dst.join(src.file_name().unwrap_or_default())
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

        let base_dst = if !dst.exists() {
            fs::create_dir_all(dst)?;
            dst.to_path_buf()
        } else if opts.content_only {
            dst.to_path_buf()
        } else {
            dst.join(src.file_name().unwrap_or_default())
        };

        if !base_dst.exists() {
            fs::create_dir_all(&base_dst)?;
        }

        let mut visited = HashSet::new();
        walk_and_copy(src, &base_dst, opts, &mut visited)?;

        return Ok(());
    }

    Err(CopyError::NotSupported(src.to_path_buf()))
}

fn walk_and_copy(
    src: &Path,
    dst: &Path,
    opts: &CopyOptions,
    visited: &mut HashSet<PathBuf>,
) -> Result<(), CopyError> {
    let real_src = src.to_path_buf();

    if !visited.insert(real_src.clone()) {
        return Err(CopyError::SymlinkLoop(real_src));
    }

    let walker = WalkDir::new(src)?.max_depth(opts.depth);
    for entry_res in walker {
        let entry = entry_res.map_err(CopyError::Walk)?;
        let src_path = entry.path();
        let rel_part = src_path.strip_prefix(src).unwrap_or(src_path);
        let dst_path = dst.join(rel_part);
        let meta = entry.symlink_metadata().map_err(CopyError::Io)?;

        if meta.is_dir() {
            if !dst_path.exists() {
                fs::create_dir_all(&dst_path)?;
            }
        } else if meta.is_file() {
            copy_one(src_path, &dst_path, opts)?;
        } else if meta.file_type().is_symlink() {
            if opts.follow_symlinks {
                let target = fs::read_link(src_path)?;
                let target_abs = if target.is_absolute() {
                    target.clone()
                } else {
                    src_path.parent().unwrap_or_else(|| Path::new("/")).join(&target)
                };

                let target_meta = target_abs.symlink_metadata().map_err(CopyError::Io)?;
                if target_meta.is_file() {
                    copy_one(&target_abs, &dst_path, opts)?;
                } else if target_meta.is_dir() {
                    walk_and_copy(&target_abs, &dst_path, opts, visited)?;
                }
            } else {
                recreate_symlink(src_path, &dst_path, opts)?;
            }
        }
    }
    visited.remove(&real_src);

    Ok(())
}


fn copy_one(src: &Path, dst: &Path, opts: &CopyOptions) -> Result<(), CopyError> {
    if dst.exists() {
        if !opts.overwrite {
            return Ok(());
        }
        fs::remove_file(dst)?;
    } else if let Some(p) = dst.parent() {
        fs::create_dir_all(p)?;
    }

    let mut input = fs::File::open(src)?;
    let mut output = fs::File::create(dst)?;
    copy(&mut input, &mut output)?;

    let mode = fs::metadata(src)?.permissions().mode();
    let mut perms = output.metadata()?.permissions();
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

    if let Some(p) = dst.parent() {
        fs::create_dir_all(p)?;
    }

    unix_fs::symlink(&target, dst)?;
    Ok(())
}

#[cfg(test)]
mod tests;