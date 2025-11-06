use super::*;

#[test]
fn it_works() {
    let cargo_dir = env!("CARGO_MANIFEST_DIR");
    let test_dir = PathBuf::from(cargo_dir).join("testdir");

    let opts = CopyOptions::default();
    let dst = Path::new("/tmp/testdir");

    match copy_recursive(test_dir.as_path(), dst, &opts) {
        Ok(sum) => println!("{:?}", sum),
        Err(e) => eprintln!("Error: {:?}", e),
    }
}