#[test]
fn parses_all_test_patterns() {
  let patterns_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/patterns");
  for entry in patterns_path.read_dir().unwrap() {
    let path = entry.unwrap().path();
    let pattern = pmaker::parse_xsd_pattern(path.clone());
    assert!(pattern.is_ok(), "Failed to parse {:?}", path);
  }
}
