#[test]
fn parses_and_saves_all_test_patterns() {
  let patterns_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/patterns");
  for entry in patterns_path.read_dir().unwrap() {
    let path = entry.unwrap().path();
    let pattern = ursa::parse_oxs_pattern(&path);
    assert!(pattern.is_ok(), "Failed to parse {:?}", path);

    let temp_path = std::env::temp_dir().join(path.file_name().unwrap());
    let result = ursa::save_oxs_pattern(&temp_path, &pattern.unwrap());
    assert!(result.is_ok(), "Failed to save {:?}", path);
  }
}
