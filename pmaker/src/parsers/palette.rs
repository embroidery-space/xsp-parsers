use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt};

use crate::PaletteItem;
use crate::parsers::xsd::read_palette_item;

enum PatternMakerPalette {
  Master,
  User,
}

impl TryFrom<Option<&std::ffi::OsStr>> for PatternMakerPalette {
  type Error = anyhow::Error;

  fn try_from(value: Option<&std::ffi::OsStr>) -> Result<Self> {
    match value {
      Some(os_str) => match os_str.to_str() {
        Some("Master") | Some("master") => Ok(PatternMakerPalette::Master),
        Some("User") | Some("user") => Ok(PatternMakerPalette::User),
        _ => Err(anyhow::anyhow!("Invalid palette type")),
      },
      None => Err(anyhow::anyhow!("No palette type provided")),
    }
  }
}

pub fn parse_palette<P: AsRef<std::path::Path>>(file_path: P) -> Result<Vec<PaletteItem>> {
  let file_path = file_path.as_ref();

  let buf = std::fs::read(file_path)?;
  let mut cursor = std::io::Cursor::new(buf);

  cursor.set_position(0x04);
  let palette_size: usize = cursor.read_u16::<LittleEndian>()?.into();

  match PatternMakerPalette::try_from(file_path.extension())? {
    PatternMakerPalette::Master => cursor.set_position(0x08),
    PatternMakerPalette::User => cursor.set_position(0x06),
  }

  let mut palette = Vec::with_capacity(palette_size);
  for _ in 0..palette_size {
    palette.push(read_palette_item(&mut cursor)?)
  }

  Ok(palette)
}
