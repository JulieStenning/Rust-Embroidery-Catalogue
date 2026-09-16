//! # Binary Embroidery File Readers
//!
//! This module provides parsing implementations for major embroidery machine file formats.
//! Each format parser implements the [`EmbroideryReader`] trait, parsing raw binary streams
//! into a unified [`EmbPattern`](crate::models::EmbPattern) model.
//!
//! ## Supported Formats
//!
//! | Format | Extension | Reader Type | Description |
//! |--------|-----------|-------------|-------------|
//! | Tajima | `.dst` | [`DstReader`] | Industry standard commercial format with 3-byte ternary coordinates. |
//! | Melco Expanded | `.exp` | [`ExpReader`] | Commercial format with 2-byte signed deltas and command escapes. |
//! | Husqvarna Viking | `.hus` | [`HusReader`] | Consumer format with compressed stitch chunks and color tables. |
//! | Janome | `.jef` | [`JefReader`] | Janome format with fixed header, thread list, and 2-byte signed offsets. |
//! | Brother / Babylock | `.pes` | [`PesReader`] | Rich Brother format supporting PEC blocks, color palettes, and metadata. |
//! | Husqvarna Pfaff | `.vp3` | [`Vp3Reader`] | Modern hierarchical format with extended color descriptions and hoops. |
//!
//! ## Coordinate & Stitch Conventions
//!
//! - **Coordinate Units:** Readers normalize all stitch delta and coordinate values into
//!   tenths of a millimetre (`0.1 mm = 1 unit`), consistent with the standard embroidery representation.
//! - **Stitch Commands:** Parsers emit standard [`StitchType`](crate::models::StitchType) commands
//!   (such as `Stitch`, `Jump`, `Trim`, `ColorChange`, `Stop`, and `End`).
//! - **Color Blocks:** When format headers supply thread color or palette information (e.g. PES, JEF, VP3, HUS),
//!   colors are populated into the pattern's thread list. Formats without embedded palettes (e.g. DST, EXP)
//!   rely on color change commands and default fallback palettes.

pub mod embroidery_reader;
pub use crate::readers::dst_reader::DstReader;
pub use crate::readers::embroidery_reader::EmbroideryReader;
pub use crate::readers::exp_reader::ExpReader;
pub use crate::readers::hus_reader::HusReader;
pub use crate::readers::jef_reader::JefReader;
pub use crate::readers::pes_reader::PesReader;
pub use crate::readers::vp3_reader::Vp3Reader;

pub mod dst_reader;
pub mod exp_reader;
pub mod hus_reader;
pub mod jef_reader;
pub mod pes_reader;
pub mod vp3_reader;
