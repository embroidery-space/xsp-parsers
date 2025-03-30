#[derive(Debug, PartialEq)]
pub struct Pattern {
  pub properties: PatternProperties,
  pub palette: (PaletteItem, Vec<PaletteItem>),
  pub fullstitches: Vec<FullStitch>,
  pub partstitches: Vec<PartStitch>,
  pub linestitches: Vec<LineStitch>,
  pub nodestitches: Vec<NodeStitch>,
}

#[derive(Debug, PartialEq)]
pub struct PatternProperties {
  pub software: String,
  pub software_version: String,
  pub height: u16,
  pub width: u16,
  pub title: String,
  pub author: String,
  pub copyright: String,
  pub instructions: String,
  pub stitches_per_inch: (u8, u8),
}

#[derive(Debug, PartialEq)]
pub struct PaletteItem {
  pub number: String,
  pub name: String,
  pub color: String,
  pub symbol: Option<Symbol>,
}

#[derive(Debug, PartialEq)]
pub enum Symbol {
  Code(u16),
  Char(char),
}

impl std::fmt::Display for Symbol {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Symbol::Code(code) => write!(f, "{}", code),
      Symbol::Char(ch) => write!(f, "{}", ch),
    }
  }
}

impl std::str::FromStr for Symbol {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if let Ok(code) = s.parse::<u16>() {
      return Ok(Symbol::Code(code));
    }

    if s.len() == 1 {
      return Ok(Symbol::Char(s.chars().next().unwrap()));
    }

    Err(anyhow::anyhow!(
      "Invalid symbol: {s}. Must be a single character or a number"
    ))
  }
}

#[derive(Debug, PartialEq)]
pub struct FullStitch {
  pub x: f32,
  pub y: f32,
  pub palindex: u8,
  pub kind: FullStitchKind,
}

#[derive(Debug, PartialEq)]
pub enum FullStitchKind {
  Full,
  Petite,
}

#[derive(Debug, PartialEq)]
pub struct PartStitch {
  pub x: f32,
  pub y: f32,
  pub palindex: u8,
  pub direction: PartStitchDirection,
  pub kind: PartStitchKind,
}

impl PartStitch {
  pub fn is_on_top_left(&self) -> bool {
    self.x.fract() < 0.5 && self.y.fract() < 0.5
  }

  pub fn is_on_top_right(&self) -> bool {
    self.x.fract() >= 0.5 && self.y.fract() < 0.5
  }

  pub fn is_on_bottom_right(&self) -> bool {
    self.x.fract() >= 0.5 && self.y.fract() >= 0.5
  }

  pub fn is_on_bottom_left(&self) -> bool {
    self.x.fract() < 0.5 && self.y.fract() >= 0.5
  }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PartStitchDirection {
  Forward = 1,
  Backward = 2,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum PartStitchKind {
  Half,
  Quarter,
}

#[derive(Debug, PartialEq)]
pub struct LineStitch {
  pub x: (f32, f32),
  pub y: (f32, f32),
  pub palindex: u8,
  pub kind: LineStitchKind,
}

#[derive(Debug, PartialEq)]
pub enum LineStitchKind {
  Back,
  Straight,
}

impl std::fmt::Display for LineStitchKind {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      LineStitchKind::Back => write!(f, "backstitch"),
      LineStitchKind::Straight => write!(f, "straightstitch"),
    }
  }
}

impl std::str::FromStr for LineStitchKind {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "backstitch" => Ok(LineStitchKind::Back),
      "straightstitch" => Ok(LineStitchKind::Straight),
      _ => Ok(LineStitchKind::Back),
    }
  }
}

#[derive(Debug, PartialEq)]
pub struct NodeStitch {
  pub x: f32,
  pub y: f32,
  pub palindex: u8,
  pub kind: NodeStitchKind,
}

#[derive(Debug, PartialEq)]
pub enum NodeStitchKind {
  FrenchKnot,
  Bead,
}

impl std::fmt::Display for NodeStitchKind {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      NodeStitchKind::FrenchKnot => write!(f, "knot"),
      NodeStitchKind::Bead => write!(f, "bead"),
    }
  }
}

impl std::str::FromStr for NodeStitchKind {
  type Err = anyhow::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    if s == "knot" {
      return Ok(NodeStitchKind::FrenchKnot);
    }

    if s.starts_with("bead") {
      return Ok(NodeStitchKind::Bead);
    }

    Err(anyhow::anyhow!("Unknown node kind: {s}"))
  }
}
