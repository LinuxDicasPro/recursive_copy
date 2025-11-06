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
    pub preserve_permissions: bool,
    pub max_depth: usize,
}

impl Default for CopyOptions {
    fn default() -> Self {
        Self {
            overwrite: true,
            follow_symlinks: false,
            create_dest: true,
            buffer_size: 8 * 1024,
            preserve_permissions: true,
            max_depth: 256,
        }
    }
}

#[derive(Debug)]
pub struct CopySummary {
    pub bytes_copied: u64,
    pub errors: usize,
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
    fn from(e: io::Error) -> Self { CopyError::Io(e) }
}

pub fn copy_recursive(src: &Path, dst: &Path, opts: &CopyOptions) -> Result<CopySummary, CopyError> {
    if !src.exists() {
        return Err(CopyError::SrcNotFound(src.to_path_buf()));
    }

    if src.is_file() {
        copy_one(src, dst, opts)?;
        return Ok(CopySummary { bytes_copied: fs::metadata(dst).map(|m| m.len()).unwrap_or(0), errors: 0 });
    }

    if !src.is_dir() {
        return Err(CopyError::NotSupported(src.to_path_buf()));
    }

    if dst.exists() && !dst.is_dir() {
        return Err(CopyError::DestNotDir(dst.to_path_buf()));
    }

    if opts.create_dest {
        fs::create_dir_all(dst)?;
    }

    let mut visited: HashSet<PathBuf> = HashSet::new();
    let mut summary = CopySummary { bytes_copied: 0, errors: 0 };

    copy_dir(src, dst, opts, 0, &mut visited, &mut summary)?;
    Ok(summary)
}

fn copy_dir(src: &Path, dst: &Path, opts: &CopyOptions, depth: usize, visited: &mut HashSet<PathBuf>, summary: &mut CopySummary) -> Result<(), CopyError> {
    if depth > opts.max_depth {
        return Err(CopyError::DepthExceeded(src.to_path_buf()));
    }

    let real_src = fs::canonicalize(src)?;
    if !visited.insert(real_src.clone()) {
        return Err(CopyError::SymlinkLoop(real_src));
    }

    for entry_res in fs::read_dir(src)? {
        let entry = match entry_res {
            Ok(e) => e,
            Err(e) => { summary.errors += 1; eprintln!("erro lendo entry: {}", e); continue; }
        };

        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(e) => { summary.errors += 1; eprintln!("erro lendo metadata: {}", e); continue; }
        };

        if meta.is_dir() {
            if !dst_path.exists() {
                fs::create_dir_all(&dst_path)?;
            }
            copy_dir(&src_path, &dst_path, opts, depth + 1, visited, summary)?;
        } else if meta.is_file() {
            match copy_one(&src_path, &dst_path, opts) {
                Ok(bytes) => {
                    summary.bytes_copied += bytes;
                }
                Err(e) => {
                    summary.errors += 1;
                    eprintln!("falha copiando {}: {:?}", src_path.display(), e);
                }
            }
        } else if meta.file_type().is_symlink() {
            if opts.follow_symlinks {
                match fs::canonicalize(&src_path) {
                    Ok(target) => {
                        if target.is_file() {
                            let bytes = copy_one(&target, &dst_path, opts)?;
                            summary.bytes_copied += bytes;
                        } else if target.is_dir() {
                            copy_dir(&target, &dst_path, opts, depth + 1, visited, summary)?;
                        }
                    }
                    Err(e) => {
                        summary.errors += 1;
                        eprintln!("erro resolvendo symlink {}: {}", src_path.display(), e);
                    }
                }
            } else {
                recreate_symlink(&src_path, &dst_path, opts, summary)?;
            }
        } else {
            summary.errors += 1;
            eprintln!("ignorado tipo especial: {}", src_path.display());
        }
    }

    Ok(())
}

fn copy_one(src: &Path, dst: &Path, opts: &CopyOptions) -> Result<u64, CopyError> {
    if dst.exists() {
        if opts.overwrite {
            fs::remove_file(dst)?;
        } else {
            return Ok(0);
        }
    } else if let Some(p) = dst.parent() {
        if opts.create_dest {
            fs::create_dir_all(p)?;
        }
    }

    let mut input = fs::File::open(src)?;
    let mut output = fs::File::create(dst)?;

    let mut buf = vec![0u8; opts.buffer_size];
    let mut total = 0u64;

    loop {
        let n = input.read(&mut buf)?;
        if n == 0 { break; }
        output.write_all(&buf[..n])?;
        total += n as u64;
    }

    if opts.preserve_permissions {
        let mode = fs::metadata(src)?.permissions().mode();
        let mut perms = fs::metadata(dst)?.permissions();
        perms.set_mode(mode);
        fs::set_permissions(dst, perms)?;
    }

    Ok(total)
}

fn recreate_symlink(src: &Path, dst: &Path, opts: &CopyOptions, summary: &mut CopySummary) -> Result<(), CopyError> {
    match fs::read_link(src) {
        Ok(target) => {
            if dst.exists() {
                match opts.overwrite {
                    true => fs::remove_file(dst)?,
                    false => return Ok(()),
                }
            }
            if let Err(e) = unix_fs::symlink(&target, dst) {
                summary.errors += 1;
                eprintln!("erro recriando symlink {}: {}", dst.display(), e);
            }
        }
        Err(e) => {
            summary.errors += 1;
            eprintln!("falha ao ler symlink {}: {}", src.display(), e);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
