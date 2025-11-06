use super::*;
use std::path::PathBuf;

#[test]
fn it_works() {
    let cargo_dir = env!("CARGO_MANIFEST_DIR");
    let test_dir = PathBuf::from(cargo_dir).join("testdir");
    let dst = Path::new("/tmp/testdir");
    let opts = CopyOptions::default();

    println!("Testing directory copy: {} → {}", test_dir.display(), dst.display());
    match copy_recursive(test_dir.as_path(), dst, &opts) {
        Ok(_) => println!("Directory copied successfully."),
        Err(e) => eprintln!("Error copying directory: {:?}", e),
    }

    let test_file = PathBuf::from(cargo_dir).join("LICENSE");
    println!("Testing file copy: {} → {}", test_file.display(), dst.display());
    match copy_recursive(test_file.as_path(), dst, &opts) {
        Ok(_) => println!("File copied successfully."),
        Err(e) => eprintln!("Error copying file: {:?}", e),
    }
}
