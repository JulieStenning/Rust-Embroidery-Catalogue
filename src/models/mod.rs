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
        Stitch { x, y, stitch_type }
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
        format!(
            "#{:02x}{:02x}{:02x}",
            self.get_red(),
            self.get_green(),
            self.get_blue()
        )
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
    pub fn add_stitch_relative(&mut self, stitch_type: StitchType, dx: f32, dy: f32) {
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

    /// Return the number of distinct thread RGB colours.
    pub fn count_distinct_thread_colors(&self) -> usize {
        self.threadlist
            .iter()
            .map(|thread| thread.color)
            .collect::<std::collections::HashSet<u32>>()
            .len()
    }
}

impl Default for EmbPattern {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── StitchType::from_command ──────────────────────────────────────

    #[test]
    fn test_from_command_known_codes() {
        // Spot-check a representative sample of the known command codes.
        assert_eq!(StitchType::from_command(0), StitchType::Stitch);
        assert_eq!(StitchType::from_command(1), StitchType::Jump);
        assert_eq!(StitchType::from_command(2), StitchType::Trim);
        assert_eq!(StitchType::from_command(3), StitchType::Stop);
        assert_eq!(StitchType::from_command(4), StitchType::End);
        assert_eq!(StitchType::from_command(5), StitchType::ColorChange);
        assert_eq!(StitchType::from_command(6), StitchType::SequinMode);
        assert_eq!(StitchType::from_command(7), StitchType::SequinEject);
        assert_eq!(StitchType::from_command(9), StitchType::NeedleSet);
        assert_eq!(StitchType::from_command(0x0B), StitchType::Slow);
        assert_eq!(StitchType::from_command(0x0C), StitchType::Fast);
        assert_eq!(
            StitchType::from_command(0x10),
            StitchType::SetChangeSequence
        );
        assert_eq!(StitchType::from_command(0xB0), StitchType::SewTo);
        assert_eq!(StitchType::from_command(0xB1), StitchType::NeedleAt);
        assert_eq!(StitchType::from_command(0xC0), StitchType::MatrixTranslate);
        assert_eq!(
            StitchType::from_command(0xC1),
            StitchType::MatrixScaleOrigin
        );
        assert_eq!(
            StitchType::from_command(0xC2),
            StitchType::MatrixRotateOrigin
        );
        assert_eq!(StitchType::from_command(0xC3), StitchType::MatrixReset);
        assert_eq!(StitchType::from_command(0xC4), StitchType::MatrixScale);
        assert_eq!(StitchType::from_command(0xC5), StitchType::MatrixRotate);
        assert_eq!(
            StitchType::from_command(0xD1),
            StitchType::ContingencyTieOnThreeSmall
        );
        assert_eq!(
            StitchType::from_command(0xD2),
            StitchType::ContingencyTieOffThreeSmall
        );
        assert_eq!(
            StitchType::from_command(0xD3),
            StitchType::ContingencyTieOnNone
        );
        assert_eq!(
            StitchType::from_command(0xD4),
            StitchType::ContingencyTieOffNone
        );
        assert_eq!(
            StitchType::from_command(0xD5),
            StitchType::OptionMaxStitchLength
        );
        assert_eq!(
            StitchType::from_command(0xD6),
            StitchType::OptionMaxJumpLength
        );
        assert_eq!(
            StitchType::from_command(0xD7),
            StitchType::OptionExplicitTrim
        );
        assert_eq!(
            StitchType::from_command(0xD8),
            StitchType::OptionImplicitTrim
        );
        assert_eq!(StitchType::from_command(0xE0), StitchType::StitchBreak);
        assert_eq!(StitchType::from_command(0xE1), StitchType::SequenceBreak);
        assert_eq!(StitchType::from_command(0xE2), StitchType::ColorBreak);
        assert_eq!(StitchType::from_command(0xE4), StitchType::TieOn);
        assert_eq!(StitchType::from_command(0xE5), StitchType::TieOff);
        assert_eq!(StitchType::from_command(0xE9), StitchType::FrameEject);
        assert_eq!(
            StitchType::from_command(0xF0),
            StitchType::ContingencyLongStitchNone
        );
        assert_eq!(
            StitchType::from_command(0xF1),
            StitchType::ContingencyLongStitchJumpNeedle
        );
        assert_eq!(
            StitchType::from_command(0xF2),
            StitchType::ContingencyLongStitchSewTo
        );
    }

    #[test]
    fn test_from_command_masks_lower_8_bits() {
        // Only the lower 8 bits matter — extra high bits are masked away.
        assert_eq!(StitchType::from_command(0x100), StitchType::Stitch);
        assert_eq!(StitchType::from_command(0x201), StitchType::Jump);
        assert_eq!(StitchType::from_command(0xABCD_0004), StitchType::End);
    }

    #[test]
    fn test_from_command_unknown() {
        // A code that isn't mapped should produce Unknown preserving the raw value.
        let result = StitchType::from_command(0xDEAD_BEEF);
        assert_eq!(result, StitchType::Unknown(0xDEAD_BEEF));
    }

    // ── StitchType::to_command ────────────────────────────────────────

    #[test]
    fn test_to_command_all_known_variants() {
        // Every known variant should produce the expected base command code.
        assert_eq!(StitchType::Stitch.to_command(), 0);
        assert_eq!(StitchType::Jump.to_command(), 1);
        assert_eq!(StitchType::Trim.to_command(), 2);
        assert_eq!(StitchType::Stop.to_command(), 3);
        assert_eq!(StitchType::End.to_command(), 4);
        assert_eq!(StitchType::ColorChange.to_command(), 5);
        assert_eq!(StitchType::SequinMode.to_command(), 6);
        assert_eq!(StitchType::SequinEject.to_command(), 7);
        assert_eq!(StitchType::NeedleSet.to_command(), 9);
        assert_eq!(StitchType::Slow.to_command(), 0x0B);
        assert_eq!(StitchType::Fast.to_command(), 0x0C);
        assert_eq!(StitchType::SetChangeSequence.to_command(), 0x10);
        assert_eq!(StitchType::SewTo.to_command(), 0xB0);
        assert_eq!(StitchType::NeedleAt.to_command(), 0xB1);
        assert_eq!(StitchType::MatrixTranslate.to_command(), 0xC0);
        assert_eq!(StitchType::MatrixScaleOrigin.to_command(), 0xC1);
        assert_eq!(StitchType::MatrixRotateOrigin.to_command(), 0xC2);
        assert_eq!(StitchType::MatrixReset.to_command(), 0xC3);
        assert_eq!(StitchType::MatrixScale.to_command(), 0xC4);
        assert_eq!(StitchType::MatrixRotate.to_command(), 0xC5);
        assert_eq!(StitchType::ContingencyTieOnThreeSmall.to_command(), 0xD1);
        assert_eq!(StitchType::ContingencyTieOffThreeSmall.to_command(), 0xD2);
        assert_eq!(StitchType::ContingencyTieOnNone.to_command(), 0xD3);
        assert_eq!(StitchType::ContingencyTieOffNone.to_command(), 0xD4);
        assert_eq!(StitchType::OptionMaxStitchLength.to_command(), 0xD5);
        assert_eq!(StitchType::OptionMaxJumpLength.to_command(), 0xD6);
        assert_eq!(StitchType::OptionExplicitTrim.to_command(), 0xD7);
        assert_eq!(StitchType::OptionImplicitTrim.to_command(), 0xD8);
        assert_eq!(StitchType::StitchBreak.to_command(), 0xE0);
        assert_eq!(StitchType::SequenceBreak.to_command(), 0xE1);
        assert_eq!(StitchType::ColorBreak.to_command(), 0xE2);
        assert_eq!(StitchType::TieOn.to_command(), 0xE4);
        assert_eq!(StitchType::TieOff.to_command(), 0xE5);
        assert_eq!(StitchType::FrameEject.to_command(), 0xE9);
        assert_eq!(StitchType::ContingencyLongStitchNone.to_command(), 0xF0);
        assert_eq!(
            StitchType::ContingencyLongStitchJumpNeedle.to_command(),
            0xF1
        );
        assert_eq!(StitchType::ContingencyLongStitchSewTo.to_command(), 0xF2);
    }

    #[test]
    fn test_to_command_unknown_and_nocommand_return_zero() {
        assert_eq!(StitchType::NoCommand.to_command(), 0);
        assert_eq!(StitchType::Unknown(42).to_command(), 0);
        assert_eq!(StitchType::Unknown(0xDEAD).to_command(), 0);
    }

    #[test]
    fn test_from_command_to_command_roundtrip() {
        // Roundtrip: from_command → to_command should recover the base code
        // for every known command value.
        let known_codes = [
            0, 1, 2, 3, 4, 5, 6, 7, 9, 0x0B, 0x0C, 0x10, 0xB0, 0xB1, 0xC0, 0xC1, 0xC2, 0xC3, 0xC4,
            0xC5, 0xD1, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xE0, 0xE1, 0xE2, 0xE4, 0xE5,
            0xE9, 0xF0, 0xF1, 0xF2,
        ];
        for &code in &known_codes {
            assert_eq!(
                StitchType::from_command(code).to_command(),
                code,
                "Roundtrip failed for code 0x{code:02X}"
            );
        }
    }

    // ── EmbPattern::default ───────────────────────────────────────────

    #[test]
    fn test_embpattern_default() {
        let pat = EmbPattern::default();
        assert!(pat.stitches.is_empty());
        assert!(pat.threadlist.is_empty());
        assert!(pat.extras.is_empty());
    }
}
