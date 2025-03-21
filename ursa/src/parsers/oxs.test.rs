use std::io::Cursor;

use super::*;

fn create_reader(xml: &str) -> Reader<&[u8]> {
  let mut reader = Reader::from_str(xml);

  let reader_config = reader.config_mut();
  reader_config.expand_empty_elements = true;
  reader_config.check_end_names = true;
  reader_config.trim_text(true);

  reader
}

fn create_writer() -> Writer<Cursor<Vec<u8>>> {
  Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2)
}

#[test]
fn reads_and_writes_pattern_properties() {
  let xml = r#"<properties oxsversion="1.0" software="MySoftware" software_version="0.0.0" chartwidth="20" chartheight="10" charttitle="My Pattern" author="Me" copyright="" instructions="Enjoy the embroidery process!" stitchesperinch="14" stitchesperinch_y="14" palettecount="5"/>"#;

  let mut reader = create_reader(xml);
  let attributes = {
    if let Event::Start(e) = reader.read_event().unwrap() {
      process_attributes(e.attributes()).unwrap()
    } else {
      unreachable!()
    }
  };

  let (palette_size, properties) = read_pattern_properties(attributes).unwrap();
  assert_eq!(palette_size, 5);
  assert_eq!(
    properties,
    PatternProperties {
      software: String::from("MySoftware"),
      software_version: String::from("0.0.0"),
      height: 10,
      width: 20,
      title: String::from("My Pattern"),
      author: String::from("Me"),
      copyright: String::from(""),
      instructions: String::from("Enjoy the embroidery process!"),
      stitches_per_inch: (14, 14),
    }
  );

  let mut writer = create_writer();
  write_pattern_properties(&mut writer, &properties, palette_size).unwrap();
  assert_eq!(xml, String::from_utf8(writer.into_inner().into_inner()).unwrap());
}

#[test]
fn reads_and_writes_palette() {
  let xml = r#"<palette>
  <palette_item index="0" number="cloth" name="cloth" color="FFFFFF"/>
  <palette_item index="1" number="DMC 310" name="Black" color="2C3225"/>
  <palette_item index="2" number="Anchor 44" name="Carmine Rose DK" color="733518" symbol="131"/>
  <palette_item index="3" number="Madeira 1206" name="Jade-MD" color="007F49" symbol="k"/>
</palette>"#;
  let palette_size = 3;

  let expected_fabric = PaletteItem {
    number: String::from("cloth"),
    name: String::from("cloth"),
    color: String::from("FFFFFF"),
    symbol: None,
  };
  let expected_palette = vec![
    PaletteItem {
      number: String::from("DMC 310"),
      name: String::from("Black"),
      color: String::from("2C3225"),
      symbol: None,
    },
    PaletteItem {
      number: String::from("Anchor 44"),
      name: String::from("Carmine Rose DK"),
      color: String::from("733518"),
      symbol: Some(Symbol::Code(131)),
    },
    PaletteItem {
      number: String::from("Madeira 1206"),
      name: String::from("Jade-MD"),
      color: String::from("007F49"),
      symbol: Some(Symbol::Char('k')),
    },
  ];

  let mut reader = create_reader(xml);
  reader.read_event().unwrap(); // Consume the start `palette` tag.
  let (fabric, palette) = read_palette(&mut reader, palette_size).unwrap();
  assert_eq!(fabric, expected_fabric);
  assert_eq!(palette, expected_palette);

  let mut writer = create_writer();
  write_palette(&mut writer, &fabric, &palette).unwrap();
  assert_eq!(xml, String::from_utf8(writer.into_inner().into_inner()).unwrap());
}

#[test]
fn reads_and_writes_full_stitches() {
  let xml = r#"<fullstitches>
  <stitch x="6" y="18" palindex="7"/>
  <stitch x="7" y="48" palindex="5"/>
  <stitch x="19" y="8" palindex="2"/>
  <stitch x="30" y="46" palindex="4"/>
</fullstitches>"#;

  let expected_stitches = vec![
    FullStitch {
      x: 6.0,
      y: 18.0,
      palindex: 6,
      kind: FullStitchKind::Full,
    },
    FullStitch {
      x: 7.0,
      y: 48.0,
      palindex: 4,
      kind: FullStitchKind::Full,
    },
    FullStitch {
      x: 19.0,
      y: 8.0,
      palindex: 1,
      kind: FullStitchKind::Full,
    },
    FullStitch {
      x: 30.0,
      y: 46.0,
      palindex: 3,
      kind: FullStitchKind::Full,
    },
  ];

  let mut reader = create_reader(xml);
  reader.read_event().unwrap(); // Consume the start `fullstitches` tag.
  let stitches = read_full_stitches(&mut reader).unwrap();
  assert_eq!(stitches, expected_stitches);

  let mut writer = create_writer();
  write_full_stitches(&mut writer, &stitches).unwrap();
  assert_eq!(xml, String::from_utf8(writer.into_inner().into_inner()).unwrap());
}

#[test]
fn reads_and_writes_part_stitches() {
  let xml = r#"<partstitches>
  <partstitch x="6" y="21" palindex1="7" palindex2="0" direction="2"/>
  <partstitch x="8" y="43" palindex1="5" palindex2="0" direction="1"/>
  <partstitch x="33" y="17" palindex1="7" palindex2="0" direction="4"/>
  <partstitch x="35" y="44" palindex1="4" palindex2="0" direction="3"/>
</partstitches>"#;

  let expected_stitches = vec![
    PartStitch {
      x: 6.0,
      y: 21.0,
      palindex: 6,
      direction: PartStitchDirection::Backward,
      kind: PartStitchKind::Quarter,
    },
    PartStitch {
      x: 8.0,
      y: 43.5,
      palindex: 4,
      direction: PartStitchDirection::Forward,
      kind: PartStitchKind::Quarter,
    },
    PartStitch {
      x: 33.0,
      y: 17.0,
      palindex: 6,
      direction: PartStitchDirection::Backward,
      kind: PartStitchKind::Half,
    },
    PartStitch {
      x: 35.0,
      y: 44.0,
      palindex: 3,
      direction: PartStitchDirection::Forward,
      kind: PartStitchKind::Half,
    },
  ];

  let mut reader = create_reader(xml);
  reader.read_event().unwrap(); // Consume the start `partstitches` tag.
  let stitches = read_part_stitches(&mut reader).unwrap();
  assert_eq!(stitches, expected_stitches);

  let mut writer = create_writer();
  write_part_stitches(&mut writer, &stitches).unwrap();
  assert_eq!(xml, String::from_utf8(writer.into_inner().into_inner()).unwrap());
}

#[test]
fn reads_and_writes_line_stitches() {
  let xml = r#"<backstitches>
  <backstitch x1="3" x2="3" y1="39" y2="40" palindex="1" objecttype="straightstitch"/>
  <backstitch x1="6" x2="7" y1="18" y2="18" palindex="2" objecttype="backstitch"/>
  <backstitch x1="7" x2="8" y1="15" y2="14" palindex="3" objecttype="straightstitch"/>
  <backstitch x1="7" x2="8" y1="54" y2="54" palindex="4" objecttype="backstitch"/>
</backstitches>"#;

  let expected_stitches = vec![
    LineStitch {
      x: (3.0, 3.0),
      y: (39.0, 40.0),
      palindex: 0,
      kind: LineStitchKind::Straight,
    },
    LineStitch {
      x: (6.0, 7.0),
      y: (18.0, 18.0),
      palindex: 1,
      kind: LineStitchKind::Back,
    },
    LineStitch {
      x: (7.0, 8.0),
      y: (15.0, 14.0),
      palindex: 2,
      kind: LineStitchKind::Straight,
    },
    LineStitch {
      x: (7.0, 8.0),
      y: (54.0, 54.0),
      palindex: 3,
      kind: LineStitchKind::Back,
    },
  ];

  let mut reader = create_reader(xml);
  reader.read_event().unwrap(); // Consume the start `linestitches` tag.
  let stitches = read_line_stitches(&mut reader).unwrap();
  assert_eq!(stitches, expected_stitches);

  let mut writer = create_writer();
  write_line_stitches(&mut writer, &stitches).unwrap();
  assert_eq!(xml, String::from_utf8(writer.into_inner().into_inner()).unwrap());
}

#[test]
fn reads_and_writes_ornaments() {
  let xml = r#"<ornaments_inc_knots_and_beads>
  <object x1="11.6875" y1="10.3125" palindex="6" objecttype="bead"/>
  <object x1="8" y1="45.1875" palindex="3" objecttype="knot"/>
</ornaments_inc_knots_and_beads>"#;

  let expected_node_stitches = vec![
    NodeStitch {
      x: 11.6875,
      y: 10.3125,
      palindex: 5,
      kind: NodeStitchKind::Bead,
    },
    NodeStitch {
      x: 8.0,
      y: 45.1875,
      palindex: 2,
      kind: NodeStitchKind::FrenchKnot,
    },
  ];

  let mut reader = create_reader(xml);
  reader.read_event().unwrap(); // Consume the start `ornaments` tag.
  let (full_stitches, node_stitches) = read_ornaments(&mut reader).unwrap();
  assert_eq!(node_stitches, expected_node_stitches);

  let mut writer = create_writer();
  write_ornaments(&mut writer, &full_stitches, &node_stitches).unwrap();
  assert_eq!(xml, String::from_utf8(writer.into_inner().into_inner()).unwrap());
}
