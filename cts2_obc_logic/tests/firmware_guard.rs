/// This file will make sure that firmware crate does not contain any tests
/// because the pipeline is not running tests from firmware crate.
/// All testable logic and tests should be moved to cts2_obc_logic crate instead
/// If we need to test firmware crate in the future, REMOVE this file and add
/// command to test firmware in the `justfile` and fix github actions `rust.yml`
/// to run tests from firmware
use std::path::Path;

fn check_directory(directory: &Path) {
    for entry in directory.read_dir().unwrap() {
        let path = entry.unwrap().path();

        if path.is_dir() {
            check_directory(&path);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            let source = std::fs::read_to_string(&path).unwrap();
            assert!(
                !source.contains("#[test]"),
                "Firmware tests are not run. Found #[test] in: {}. \
                 Move testable logic and tests to cts2_obc_logic.",
                path.display()
            );
        }
    }
}

#[test]
fn firmware_has_no_tests() {
    let firmware = Path::new(env!("CARGO_MANIFEST_DIR")).join("../cts2_obc_firmware");
    check_directory(&firmware);
}
