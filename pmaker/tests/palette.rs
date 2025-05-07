#[test]
fn parses_all_test_palettes() {
  let palettes_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/palettes");
  for entry in palettes_path.read_dir().unwrap() {
    let path = entry.unwrap().path();
    let palette = pmaker::parse_palette(path.clone());
    assert!(palette.is_ok(), "Failed to parse {:?}", path);
  }
}
