use super::*;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

const SYMLINKS: bool = false;
const COPY_ONLY: bool = false;
const NO_DEST: bool = false;

fn create_file(path: &PathBuf, content: &str) {
    if let Some(p) = path.parent() {
        fs::create_dir_all(p).unwrap();
    }
    let mut file = File::create(path).unwrap();
    writeln!(file, "{}", content).unwrap();
}

fn cleanup(path: &PathBuf) {
    if path.exists() {
        fs::remove_dir_all(path).ok();
    }
}


#[test]
fn test_copy_recursive_with_symlinks() {
    let base = PathBuf::from("/tmp/recursive_copy_test_symlinks");
    let src = base.join("src");
    let dst = base.join("dst");

    cleanup(&base);

    let create = NO_DEST;
    if !create {
        fs::create_dir_all(&dst).unwrap();
    }

    fs::create_dir_all(src.join("subdir/nested")).unwrap();
    create_file(&src.join("root.txt"), "Root file");
    create_file(&src.join("subdir/file1.txt"), "File in subdir");
    create_file(&src.join("subdir/nested/deep.txt"), "Nested file");

    let symlink_file = src.join("link_to_root");
    let symlink_dir = src.join("link_to_nested");
    std::os::unix::fs::symlink("root.txt", &symlink_file).unwrap_or_default();
    std::os::unix::fs::symlink("subdir/nested", &symlink_dir).unwrap_or_default();

    let mut opts = CopyOptions::default();
    opts.follow_symlinks = SYMLINKS;
    opts.content_only = COPY_ONLY;

    let mut final_dst = PathBuf::from(&dst);
    if !opts.content_only && !create {
        final_dst = dst.join("src");
    }

    println!("--- Running Test: Recursive Copy ---");

    copy_recursive(&src, &dst, &opts).expect("Copy failed");

    let files = [
        "root.txt",
        "subdir/file1.txt",
        "subdir/nested/deep.txt",
        "link_to_root",
        "link_to_nested"
    ];

    for f in files {
        let path = final_dst.join(f);
        assert!(path.exists());
        let symlink_info = if path.is_symlink() { " (symlink)" } else { "" };
        println!("  [OK] Copied file exists{}: {}", symlink_info, path.display());
    }
}

#[test]
fn test_copy_single_file_to_existing_dir() {
    let base = PathBuf::from("/tmp/test_single_file");
    let src_file = base.join("source_file.txt");
    let dst_dir = base.join("dest_dir");
    let expected_dst_file = dst_dir.join("source_file.txt");

    cleanup(&base);

    fs::create_dir_all(&dst_dir).unwrap();
    create_file(&src_file, "This is the content of the file.");

    let opts = CopyOptions::default();

    println!("--- Running Test: Single File Copy ---");
    copy_recursive(&src_file, &dst_dir, &opts).expect("File copy failed");

    assert!(dst_dir.is_dir(), "The destination must be a directory.");
    assert!(expected_dst_file.exists(), "The copied file does not exist in the expected destination.");
    println!("  [OK] File copied successfully to directory: {}", expected_dst_file.display());

    let new_file_name = base.join("new_file_name.txt");
    copy_recursive(&src_file, &new_file_name, &opts).expect("File copy failed to new name");

    assert!(new_file_name.exists(), "The file should have been copied and renamed.");
    println!("  [OK] File copied successfully with rename: {}", new_file_name.display());
}