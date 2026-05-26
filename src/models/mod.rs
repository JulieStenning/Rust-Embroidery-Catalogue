/// Represents the type of a stitch or machine command.
/// The integer values correspond to the EmbConstant command codes
/// used by pyembroidery.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StitchType {
    NoCommand,
    Stitch,
    Jump,
    Trim,
    Stop,
    End,
    ColorChange,
    NeedleSet,
    SequinMode,
    SequinEject,
    Slow,
    Fast,
    SetChangeSequence,
    SewTo,
    NeedleAt,
    StitchBreak,
    SequenceBreak,
    ColorBreak,
    TieOn,
    TieOff,
    FrameEject,
    MatrixTranslate,
    MatrixScaleOrigin,
    MatrixRotateOrigin,
    MatrixScale,
    MatrixRotate,
    MatrixReset,
    OptionMaxStitchLength,
    OptionMaxJumpLength,
    OptionExplicitTrim,
    OptionImplicitTrim,
    ContingencyTieOnNone,
    ContingencyTieOnThreeSmall,
    ContingencyTieOffNone,
    ContingencyTieOffThreeSmall,
    ContingencyLongStitchNone,
    ContingencyLongStitchJumpNeedle,
    ContingencyLongStitchSewTo,
    /// A command code not covered by the known variants.
    /// Stores the raw u32 command value.
    Unknown(u32),
}

impl StitchType {
    /// Convert from the raw command integer (as used in pyembroidery).
    pub fn from_command(cmd: u32) -> Self {
        let base = cmd & 0x0000_00FF;
        match base {
            0 => StitchType::Stitch,
            1 => StitchType::Jump,
            2 => StitchType::Trim,
            3 => StitchType::Stop,
            4 => StitchType::End,
            5 => StitchType::ColorChange,
            6 => StitchType::SequinMode,
            7 => StitchType::SequinEject,
            9 => StitchType::NeedleSet,
            0x0B => StitchType::Slow,
            0x0C => StitchType::Fast,
            0x10 => StitchType::SetChangeSequence,
            0xB0 => StitchType::SewTo,
            0xB1 => StitchType::NeedleAt,
            0xC0 => StitchType::MatrixTranslate,
            0xC1 => StitchType::MatrixScaleOrigin,
            0xC2 => StitchType::MatrixRotateOrigin,
            0xC3 => StitchType::MatrixReset,
            0xC4 => StitchType::MatrixScale,
            0xC5 => StitchType::MatrixRotate,
            0xD1 => StitchType::ContingencyTieOnThreeSmall,
            0xD2 => StitchType::ContingencyTieOffThreeSmall,
            0xD3 => StitchType::ContingencyTieOnNone,
            0xD4 => StitchType::ContingencyTieOffNone,
            0xD5 => StitchType::OptionMaxStitchLength,
            0xD6 => StitchType::OptionMaxJumpLength,
            0xD7 => StitchType::OptionExplicitTrim,
            0xD8 => StitchType::OptionImplicitTrim,
            0xE0 => StitchType::StitchBreak,
            0xE1 => StitchType::SequenceBreak,
            0xE2 => StitchType::ColorBreak,
            0xE4 => StitchType::TieOn,
            0xE5 => StitchType::TieOff,
            0xE9 => StitchType::FrameEject,
            0xF0 => StitchType::ContingencyLongStitchNone,
            0xF1 => StitchType::ContingencyLongStitchJumpNeedle,
            0xF2 => StitchType::ContingencyLongStitchSewTo,
            _ => StitchType::Unknown(cmd),
        }
    }

    /// Convert this stitch type back to its base command code (lower 8 bits).
    pub fn to_command(&self) -> u32 {
        match self {
            StitchType::Stitch => 0,
            StitchType::Jump => 1,
            StitchType::Trim => 2,
            StitchType::Stop => 3,
            StitchType::End => 4,
            StitchType::ColorChange => 5,
            StitchType::SequinMode => 6,
            StitchType::SequinEject => 7,
            StitchType::NeedleSet => 9,
            StitchType::Slow => 0x0B,
            StitchType::Fast => 0x0C,
            StitchType::SetChangeSequence => 0x10,
            StitchType::SewTo => 0xB0,
            StitchType::NeedleAt => 0xB1,
            StitchType::MatrixTranslate => 0xC0,
            StitchType::MatrixScaleOrigin => 0xC1,
            StitchType::MatrixRotateOrigin => 0xC2,
            StitchType::MatrixReset => 0xC3,
            StitchType::MatrixScale => 0xC4,
            StitchType::MatrixRotate => 0xC5,
            StitchType::ContingencyTieOnThreeSmall => 0xD1,
            StitchType::ContingencyTieOffThreeSmall => 0xD2,
            StitchType::ContingencyTieOnNone => 0xD3,
            StitchType::ContingencyTieOffNone => 0xD4,
            StitchType::OptionMaxStitchLength => 0xD5,
            StitchType::OptionMaxJumpLength => 0xD6,
            StitchType::OptionExplicitTrim => 0xD7,
            StitchType::OptionImplicitTrim => 0xD8,
            StitchType::StitchBreak => 0xE0,
            StitchType::SequenceBreak => 0xE1,
            StitchType::ColorBreak => 0xE2,
            StitchType::TieOn => 0xE4,
            StitchType::TieOff => 0xE5,
            StitchType::FrameEject => 0xE9,
            StitchType::ContingencyLongStitchNone => 0xF0,
            StitchType::ContingencyLongStitchJumpNeedle => 0xF1,
            StitchType::ContingencyLongStitchSewTo => 0xF2,
            StitchType::NoCommand | StitchType::Unknown(_) => 0,
        }
    }
}

/// A single stitch or machine command at a specific position.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Stitch {
    pub x: f32,
    pub y: f32,
    pub stitch_type: StitchType,
}

impl Stitch {
    pub fn new(x: f32, y: f32, stitch_type: StitchType) -> Self {
        Stitch {
            x,
            y,
            stitch_type,
        }
    }
}

/// Represents an embroidery thread colour with its metadata.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EmbThread {
    /// RGB colour value (e.g. 0xFF0000 for red).
    pub color: u32,
    pub description: Option<String>,
    pub catalog_number: Option<String>,
    pub details: Option<String>,
    pub brand: Option<String>,
    pub chart: Option<String>,
    pub weight: Option<String>,
}

impl EmbThread {
    pub fn new(color: u32) -> Self {
        EmbThread {
            color,
            description: None,
            catalog_number: None,
            details: None,
            brand: None,
            chart: None,
            weight: None,
        }
    }

    /// Get the red component (0-255).
    pub fn get_red(&self) -> u8 {
        ((self.color >> 16) & 0xFF) as u8
    }

    /// Get the green component (0-255).
    pub fn get_green(&self) -> u8 {
        ((self.color >> 8) & 0xFF) as u8
    }

    /// Get the blue component (0-255).
    pub fn get_blue(&self) -> u8 {
        (self.color & 0xFF) as u8
    }

    /// Return the hex colour string e.g. "#ff0000".
    pub fn hex_color(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.get_red(), self.get_green(), self.get_blue())
    }
}

/// The core embroidery pattern, holding all stitches and thread colours.
#[derive(Debug, Clone, PartialEq)]
pub struct EmbPattern {
    pub stitches: Vec<Stitch>,
    pub threadlist: Vec<EmbThread>,
    pub extras: std::collections::HashMap<String, String>,
}

impl EmbPattern {
    pub fn new() -> Self {
        EmbPattern {
            stitches: Vec::new(),
            threadlist: Vec::new(),
            extras: std::collections::HashMap::new(),
        }
    }

    /// Add a stitch with an absolute position.
    pub fn add_stitch_absolute(&mut self, stitch_type: StitchType, x: f32, y: f32) {
        self.stitches.push(Stitch::new(x, y, stitch_type));
    }

    /// Add a stitch relative to the last stitch position.
    pub fn add_stitch_relative(
        &mut self,
        stitch_type: StitchType,
        dx: f32,
        dy: f32,
    ) {
        let (prev_x, prev_y) = self
            .stitches
            .last()
            .map(|s| (s.x, s.y))
            .unwrap_or((0.0, 0.0));
        self.add_stitch_absolute(stitch_type, prev_x + dx, prev_y + dy);
    }

    /// Add a thread to the thread list.
    pub fn add_thread(&mut self, thread: EmbThread) {
        self.threadlist.push(thread);
    }

    /// Returns the bounding box of all stitches: (min_x, min_y, max_x, max_y).
    pub fn bounds(&self) -> (f32, f32, f32, f32) {
        let mut min_x = f32::INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for stitch in &self.stitches {
            if stitch.x < min_x {
                min_x = stitch.x;
            }
            if stitch.x > max_x {
                max_x = stitch.x;
            }
            if stitch.y < min_y {
                min_y = stitch.y;
            }
            if stitch.y > max_y {
                max_y = stitch.y;
            }
        }

        (min_x, min_y, max_x, max_y)
    }

    /// Count the number of stitches of a specific type.
    pub fn count_stitch_commands(&self, stitch_type: StitchType) -> usize {
        self.stitches
            .iter()
            .filter(|s| s.stitch_type == stitch_type)
            .count()
    }

    /// Count the number of colour changes.
    pub fn count_color_changes(&self) -> usize {
        self.count_stitch_commands(StitchType::ColorChange)
    }

    /// Return the total number of stitches.
    pub fn count_stitches(&self) -> usize {
        self.stitches.len()
    }

    /// Return the number of threads.
    pub fn count_threads(&self) -> usize {
        self.threadlist.len()
    }
}

impl Default for EmbPattern {
    fn default() -> Self {
        Self::new()
    }
}