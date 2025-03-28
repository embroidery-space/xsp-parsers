use std::io;
use std::str::FromStr;

use anyhow::Result;
use quick_xml::events::{BytesDecl, Event};
use quick_xml::{Reader, Writer};

use super::{AttributesMap, process_attributes};
use crate::schemas::oxs::*;

#[cfg(test)]
#[path = "oxs.test.rs"]
mod tests;

pub fn parse_oxs_pattern<P: AsRef<std::path::Path>>(file_path: P) -> Result<Pattern> {
  let mut reader = Reader::from_file(file_path.as_ref())?;
  let reader_config = reader.config_mut();
  reader_config.expand_empty_elements = true;
  reader_config.check_end_names = true;
  reader_config.trim_text(true);

  let mut palette_size = None;

  let mut properties = None;
  let mut palette = None;
  let mut fullstitches = Vec::new();
  let mut partstitches = Vec::new();
  let mut linestitches = Vec::new();
  let mut nodestitches = Vec::new();

  let mut buf = Vec::new();
  loop {
    match reader.read_event_into(&mut buf) {
      Ok(Event::Start(ref e)) => match e.name().as_ref() {
        b"properties" => {
          let attributes = process_attributes(e.attributes())?;
          let (palsize, props) = read_pattern_properties(attributes)?;
          palette_size = Some(palsize);
          properties = Some(props);
        }
        b"palette" => {
          if let Some(palette_size) = palette_size {
            palette = Some(read_palette(&mut reader, palette_size)?);
          } else {
            anyhow::bail!("Palette size is not set or the pattern properties are not read yet");
          }
        }
        b"fullstitches" => fullstitches.extend(read_full_stitches(&mut reader)?),
        b"partstitches" => partstitches.extend(read_part_stitches(&mut reader)?),
        b"backstitches" => linestitches.extend(read_line_stitches(&mut reader)?),
        b"ornaments_inc_knots_and_beads" => {
          let (fulls, nodes) = read_ornaments(&mut reader)?;
          fullstitches.extend(fulls);
          nodestitches.extend(nodes);
        }
        _ => {}
      },
      Ok(Event::End(ref e)) if e.name().as_ref() == b"chart" => break,
      // We don't expect to receive EOF here, because we should have found the end of the `chart` tag.
      Ok(Event::Eof) => anyhow::bail!("Unexpected EOF"),
      Err(e) => anyhow::bail!("Error at position {}: {e:?}", reader.error_position()),
      _ => {}
    }
    buf.clear();
  }

  if properties.is_none() || palette.is_none() {
    anyhow::bail!("Pattern properties or palette are not found");
  }

  Ok(Pattern {
    properties: properties.unwrap(),
    palette: palette.unwrap(),
    fullstitches,
    partstitches,
    linestitches,
    nodestitches,
  })
}

pub fn save_oxs_pattern<P: AsRef<std::path::Path>>(file_path: P, pattern: &Pattern) -> io::Result<()> {
  let mut file = std::fs::OpenOptions::new()
    .create(true)
    .write(true)
    .truncate(true)
    .open(file_path.as_ref())?;
  save_pattern_inner(&mut file, pattern)
}

pub fn save_oxs_pattern_to_vec(pattern: &Pattern) -> Result<Vec<u8>> {
  let mut buf = Vec::new();
  save_pattern_inner(&mut buf, pattern)?;
  Ok(buf)
}

fn save_pattern_inner<W: io::Write>(writer: &mut W, pattern: &Pattern) -> io::Result<()> {
  // In the development mode, we want to have a pretty-printed XML file for easy debugging.
  #[cfg(debug_assertions)]
  let mut writer = Writer::new_with_indent(writer, b' ', 2);
  #[cfg(not(debug_assertions))]
  let mut writer = Writer::new(writer);

  writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))?;
  writer.create_element("chart").write_inner_content(|writer| {
    let (fabric, palette) = &pattern.palette;

    write_format(writer)?;
    write_pattern_properties(writer, &pattern.properties, palette.len())?;
    write_palette(writer, fabric, palette)?;
    write_full_stitches(writer, &pattern.fullstitches)?;
    write_part_stitches(writer, &pattern.partstitches)?;
    write_line_stitches(writer, &pattern.linestitches)?;
    write_ornaments(writer, &pattern.fullstitches, &pattern.nodestitches)?;

    Ok(())
  })?;

  Ok(())
}

fn write_format<W: io::Write>(writer: &mut Writer<W>) -> io::Result<()> {
  writer
    .create_element("format")
    .with_attributes([
      ("comments01","Designed to allow interchange of basic pattern data between any cross stitch style software"),
      ("comments02","the 'properties' section establishes size, copyright, authorship and software used"),
      ("comments03","The features of each software package varies, but using XML each can pick out the things it can deal with, while ignoring others"),
      ("comments04","The basic items are :"),
      ("comments05","'palette'..a set of colors used in the design: palettecount excludes cloth color, which is item 0"),
      ("comments06","'fullstitches'.. simple crosses"),
      ("comments07","'backstitches'.. lines/objects with a start and end point"),
      ("comments08","(There is a wide variety of ways of treating part stitches, knots, beads and so on.)"),
      ("comments09","Colors are expressed in hex RGB format."),
      ("comments10","Decimal numbers use US/UK format where '.' is the indicator - eg 0.5 is 'half'"),
      ("comments11","For readability, please use words not enumerations"),
      ("comments12","The properties, fullstitches, and backstitches elements should be considered mandatory, even if empty"),
      ("comments13","element and attribute names are always lowercase"),
    ])
    .write_empty()?;
  Ok(())
}

fn read_pattern_properties(attributes: AttributesMap) -> Result<(usize, PatternProperties)> {
  Ok((
    attributes.get("palettecount").unwrap().parse::<usize>()?,
    PatternProperties {
      software: attributes.get("software").unwrap().to_owned(),
      software_version: attributes.get("software_version").unwrap().to_owned(),
      width: attributes.get("chartwidth").unwrap().parse()?,
      height: attributes.get("chartheight").unwrap().parse()?,
      title: attributes.get("charttitle").unwrap_or(&String::new()).to_owned(),
      author: attributes.get("author").unwrap_or(&String::new()).to_owned(),
      copyright: attributes.get("copyright").unwrap_or(&String::new()).to_owned(),
      instructions: attributes.get("instructions").unwrap_or(&String::new()).to_owned(),
      stitches_per_inch: (
        attributes.get("stitchesperinch").unwrap().parse()?,
        attributes.get("stitchesperinch_y").unwrap().parse()?,
      ),
    },
  ))
}

fn write_pattern_properties<W: io::Write>(
  writer: &mut Writer<W>,
  properties: &PatternProperties,
  palette_size: usize,
) -> io::Result<()> {
  writer
    .create_element("properties")
    .with_attributes([
      ("oxsversion", "1.0"),
      ("software", properties.software.as_str()),
      ("software_version", properties.software_version.to_string().as_str()),
      ("chartwidth", properties.width.to_string().as_str()),
      ("chartheight", properties.height.to_string().as_str()),
      ("charttitle", properties.title.as_str()),
      ("author", properties.author.as_str()),
      ("copyright", properties.copyright.as_str()),
      ("instructions", properties.instructions.as_str()),
      ("stitchesperinch", properties.stitches_per_inch.0.to_string().as_str()),
      ("stitchesperinch_y", properties.stitches_per_inch.1.to_string().as_str()),
      ("palettecount", palette_size.to_string().as_str()),
    ])
    .write_empty()?;
  Ok(())
}

fn read_palette<R: io::BufRead>(
  reader: &mut Reader<R>,
  palette_size: usize,
) -> Result<(PaletteItem, Vec<PaletteItem>)> {
  let mut buf = Vec::new();
  let fabric = if let Event::Start(ref e) = reader.read_event_into(&mut buf)? {
    let attributes = process_attributes(e.attributes())?;
    PaletteItem {
      number: attributes.get("number").unwrap().to_owned(),
      name: attributes.get("name").unwrap().to_owned(),
      color: attributes.get("color").unwrap().to_owned(),
      symbol: None,
    }
  } else {
    anyhow::bail!("Expected a start tag for the fabric palette item");
  };
  reader.read_event_into(&mut buf)?; // end of the fabric palette item tag

  let mut palette = Vec::with_capacity(palette_size);
  for _ in 0..palette_size {
    buf.clear();
    if let Event::Start(ref e) = reader.read_event_into(&mut buf)? {
      let attributes = process_attributes(e.attributes())?;

      palette.push(PaletteItem {
        number: attributes.get("number").unwrap().to_owned(),
        name: attributes.get("name").unwrap().to_owned(),
        color: attributes.get("color").unwrap().to_owned(),
        symbol: {
          if let Some(symbol) = attributes.get("symbol") {
            Some(symbol.parse()?)
          } else {
            None
          }
        },
      });

      // Skip the rest of the palette item tag.
      reader.read_to_end_into(e.to_end().name(), &mut Vec::new())?;
    } else {
      anyhow::bail!("Expected a start tag for the palette item");
    }
  }

  Ok((fabric, palette))
}

fn write_palette<W: io::Write>(
  writer: &mut Writer<W>,
  fabric: &PaletteItem,
  palette: &[PaletteItem],
) -> io::Result<()> {
  writer.create_element("palette").write_inner_content(|writer| {
    writer
      .create_element("palette_item")
      .with_attributes([
        ("index", "0"),
        ("number", fabric.number.as_str()),
        ("name", fabric.name.as_str()),
        ("color", fabric.color.as_str()),
      ])
      .write_empty()?;

    for (index, palitem) in palette.iter().enumerate() {
      let index = (index + 1).to_string();
      let mut attributes = vec![
        ("index", index.as_str()),
        ("number", palitem.number.as_str()),
        ("name", palitem.name.as_str()),
        ("color", palitem.color.as_str()),
      ];

      let symbol;
      if let Some(s) = &palitem.symbol {
        symbol = s.to_string();
        attributes.push(("symbol", symbol.as_str()));
      }

      writer
        .create_element("palette_item")
        .with_attributes(attributes)
        .write_empty()?;
    }

    Ok(())
  })?;

  Ok(())
}

fn read_full_stitches<R: io::BufRead>(reader: &mut Reader<R>) -> Result<Vec<FullStitch>> {
  let mut full_stitches = Vec::new();

  let mut buf = Vec::new();
  loop {
    match reader.read_event_into(&mut buf)? {
      Event::Start(ref e) if e.name().as_ref() == b"stitch" => {
        let attributes = process_attributes(e.attributes())?;
        full_stitches.push(FullStitch {
          x: attributes.get("x").unwrap().parse()?,
          y: attributes.get("y").unwrap().parse()?,
          palindex: attributes.get("palindex").unwrap().parse::<usize>()? - 1,
          kind: FullStitchKind::Full,
        });
      }
      Event::End(ref e) if e.name().as_ref() == b"fullstitches" => break,
      _ => {}
    }

    buf.clear();
  }

  Ok(full_stitches)
}

fn write_full_stitches<W: io::Write>(writer: &mut Writer<W>, full_stitches: &[FullStitch]) -> io::Result<()> {
  writer.create_element("fullstitches").write_inner_content(|writer| {
    for full_stitch in full_stitches.iter().filter(|fs| fs.kind == FullStitchKind::Full) {
      writer
        .create_element("stitch")
        .with_attributes([
          ("x", full_stitch.x.to_string().as_str()),
          ("y", full_stitch.y.to_string().as_str()),
          ("palindex", (full_stitch.palindex + 1).to_string().as_str()),
        ])
        .write_empty()?;
    }

    Ok(())
  })?;

  Ok(())
}

fn read_part_stitches<R: io::BufRead>(reader: &mut Reader<R>) -> Result<Vec<PartStitch>> {
  let mut part_stitches = Vec::new();

  let mut buf = Vec::new();
  loop {
    match reader.read_event_into(&mut buf)? {
      Event::Start(ref e) if e.name().as_ref() == b"partstitch" => {
        let attributes = process_attributes(e.attributes())?;

        let x = attributes.get("x").unwrap().parse()?;
        let y = attributes.get("y").unwrap().parse()?;

        let direction_value: u8 = attributes.get("direction").unwrap().parse()?;
        let direction = match direction_value {
          1 | 3 => PartStitchDirection::Forward,
          2 | 4 => PartStitchDirection::Backward,
          _ => panic!("Unknown part stitch direction"),
        };
        let kind = match direction_value {
          1 | 2 => PartStitchKind::Quarter,
          3 | 4 => PartStitchKind::Half,
          _ => panic!("Unknown part stitch kind"),
        };

        let palindex1: usize = attributes.get("palindex1").unwrap().parse()?;
        let palindex2: usize = attributes.get("palindex2").unwrap().parse()?;

        if palindex1 != 0 {
          let (x, y) = if direction_value == 1 { (x, y + 0.5) } else { (x, y) };
          part_stitches.push(PartStitch {
            x,
            y,
            palindex: palindex1 - 1,
            kind,
            direction,
          });
        }

        if palindex2 != 0 {
          let (x, y) = if direction_value == 1 {
            (x + 0.5, y)
          } else if direction_value == 2 {
            (x + 0.5, y + 0.5)
          } else {
            (x, y)
          };
          part_stitches.push(PartStitch {
            x,
            y,
            palindex: palindex2 - 1,
            kind,
            direction,
          });
        }
      }
      Event::End(ref e) if e.name().as_ref() == b"partstitches" => break,
      _ => {}
    }

    buf.clear();
  }

  Ok(part_stitches)
}

fn write_part_stitches<W: io::Write>(writer: &mut Writer<W>, part_stitches: &[PartStitch]) -> io::Result<()> {
  writer.create_element("partstitches").write_inner_content(|writer| {
    let mut seen_quarters = std::collections::HashSet::new();
    for part_stitch in part_stitches.iter() {
      let (palindex1, palindex2) = match part_stitch.kind {
        PartStitchKind::Half => (part_stitch.palindex + 1, 0),
        PartStitchKind::Quarter => {
          if seen_quarters.contains(&(part_stitch.x.to_string(), part_stitch.y.to_string())) {
            continue;
          }

          seen_quarters.insert((part_stitch.x.to_string(), part_stitch.y.to_string()));

          let mut palindexes = (0, 0);

          match part_stitch.direction {
            PartStitchDirection::Forward => {
              if part_stitch.is_on_bottom_left() {
                palindexes.0 = part_stitch.palindex + 1;
              } else if let Some(part_stitch) = part_stitches.iter().find(|&ps| {
                ps == &PartStitch {
                  x: part_stitch.x.floor(),
                  y: part_stitch.y + 0.5,
                  ..*part_stitch
                }
              }) {
                seen_quarters.insert((part_stitch.x.to_string(), part_stitch.y.to_string()));
                palindexes.0 = part_stitch.palindex + 1;
              }

              if part_stitch.is_on_top_right() {
                palindexes.1 = part_stitch.palindex + 1;
              } else if let Some(part_stitch) = part_stitches.iter().find(|&ps| {
                ps == &PartStitch {
                  x: part_stitch.x + 0.5,
                  y: part_stitch.y.floor(),
                  ..*part_stitch
                }
              }) {
                seen_quarters.insert((part_stitch.x.to_string(), part_stitch.y.to_string()));
                palindexes.1 = part_stitch.palindex + 1;
              }
            }

            PartStitchDirection::Backward => {
              if part_stitch.is_on_top_left() {
                palindexes.0 = part_stitch.palindex + 1;
              } else if let Some(part_stitch) = part_stitches.iter().find(|&ps| {
                ps == &PartStitch {
                  x: part_stitch.x.floor(),
                  y: part_stitch.y.floor(),
                  ..*part_stitch
                }
              }) {
                seen_quarters.insert((part_stitch.x.to_string(), part_stitch.y.to_string()));
                palindexes.0 = part_stitch.palindex + 1;
              }

              if part_stitch.is_on_bottom_right() {
                palindexes.1 = part_stitch.palindex + 1;
              } else if let Some(part_stitch) = part_stitches.iter().find(|&ps| {
                ps == &PartStitch {
                  x: part_stitch.x + 0.5,
                  y: part_stitch.y + 0.5,
                  ..*part_stitch
                }
              }) {
                seen_quarters.insert((part_stitch.x.to_string(), part_stitch.y.to_string()));
                palindexes.1 = part_stitch.palindex + 1;
              }
            }
          };

          palindexes
        }
      };
      let direction = match part_stitch.kind {
        PartStitchKind::Half => part_stitch.direction as u8 + 2,
        PartStitchKind::Quarter => part_stitch.direction as u8,
      };
      println!("direction: {:?}, kind: {:?}", direction, part_stitch.kind);

      writer
        .create_element("partstitch")
        .with_attributes([
          ("x", part_stitch.x.trunc().to_string().as_str()),
          ("y", part_stitch.y.trunc().to_string().as_str()),
          ("palindex1", palindex1.to_string().as_str()),
          ("palindex2", palindex2.to_string().as_str()),
          ("direction", direction.to_string().as_str()),
        ])
        .write_empty()?;
    }

    Ok(())
  })?;

  Ok(())
}

fn read_line_stitches<R: io::BufRead>(reader: &mut Reader<R>) -> Result<Vec<LineStitch>> {
  let mut line_stitches = Vec::new();

  let mut buf = Vec::new();
  loop {
    match reader.read_event_into(&mut buf)? {
      Event::Start(ref e) if e.name().as_ref() == b"backstitch" => {
        let attributes = process_attributes(e.attributes())?;
        line_stitches.push(LineStitch {
          x: (
            attributes.get("x1").unwrap().parse()?,
            attributes.get("x2").unwrap().parse()?,
          ),
          y: (
            attributes.get("y1").unwrap().parse()?,
            attributes.get("y2").unwrap().parse()?,
          ),
          palindex: attributes.get("palindex").unwrap().parse::<usize>()? - 1,
          kind: attributes.get("objecttype").unwrap().parse()?,
        });
      }
      Event::End(ref e) if e.name().as_ref() == b"backstitches" => break,
      _ => {}
    }

    buf.clear();
  }

  Ok(line_stitches)
}

fn write_line_stitches<W: io::Write>(writer: &mut Writer<W>, line_stitches: &[LineStitch]) -> io::Result<()> {
  writer.create_element("backstitches").write_inner_content(|writer| {
    for line_stitch in line_stitches.iter() {
      writer
        .create_element("backstitch")
        .with_attributes([
          ("x1", line_stitch.x.0.to_string().as_str()),
          ("x2", line_stitch.x.1.to_string().as_str()),
          ("y1", line_stitch.y.0.to_string().as_str()),
          ("y2", line_stitch.y.1.to_string().as_str()),
          ("palindex", (line_stitch.palindex + 1).to_string().as_str()),
          ("objecttype", line_stitch.kind.to_string().as_str()),
        ])
        .write_empty()?;
    }

    Ok(())
  })?;

  Ok(())
}

fn read_ornaments<R: io::BufRead>(reader: &mut Reader<R>) -> Result<(Vec<FullStitch>, Vec<NodeStitch>)> {
  let mut full_stitches = Vec::new();
  let mut node_stitches = Vec::new();

  let mut buf = Vec::new();
  loop {
    match reader.read_event_into(&mut buf)? {
      Event::Start(ref e) if e.name().as_ref() == b"object" => {
        let attributes = process_attributes(e.attributes())?;

        let x = attributes.get("x1").unwrap().parse()?;
        let y = attributes.get("y1").unwrap().parse()?;
        let palindex = attributes.get("palindex").unwrap().parse::<usize>()? - 1;
        let kind = attributes.get("objecttype").unwrap();

        // Yes, the Ursa Software's OXS format uses the "quarter" stitch for petites.
        if kind == "quarter" {
          full_stitches.push(FullStitch {
            x,
            y,
            palindex,
            kind: FullStitchKind::Petite,
          });
        }

        if kind.starts_with("bead") || kind == "knot" {
          node_stitches.push(NodeStitch {
            x,
            y,
            palindex,
            kind: NodeStitchKind::from_str(kind).unwrap(),
          });
        }
      }
      Event::End(ref e) if e.name().as_ref() == b"ornaments_inc_knots_and_beads" => break,
      _ => {}
    }

    buf.clear();
  }

  Ok((full_stitches, node_stitches))
}

fn write_ornaments<W: io::Write>(
  writer: &mut Writer<W>,
  full_stitches: &[FullStitch],
  node_stitches: &[NodeStitch],
) -> io::Result<()> {
  writer
    .create_element("ornaments_inc_knots_and_beads")
    .write_inner_content(|writer| {
      for full_stitch in full_stitches.iter().filter(|fs| fs.kind == FullStitchKind::Petite) {
        writer
          .create_element("object")
          .with_attributes([
            ("x1", full_stitch.x.to_string().as_str()),
            ("y1", full_stitch.y.to_string().as_str()),
            ("palindex", (full_stitch.palindex + 1).to_string().as_str()),
            ("objecttype", "quarter"),
          ])
          .write_empty()?;
      }

      for node_stitch in node_stitches.iter() {
        writer
          .create_element("object")
          .with_attributes([
            ("x1", node_stitch.x.to_string().as_str()),
            ("y1", node_stitch.y.to_string().as_str()),
            ("palindex", (node_stitch.palindex + 1).to_string().as_str()),
            ("objecttype", node_stitch.kind.to_string().as_str()),
          ])
          .write_empty()?;
      }

      Ok(())
    })?;

  Ok(())
}
