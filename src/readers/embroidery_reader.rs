
/// Trait for embroidery file readers.
///
/// All implementations **must** populate the returned `EmbPattern` with enough information
/// for rendering a preview image using the PNG renderer. This means:
/// - The `stitches` vector must contain all stitch positions and commands.
/// - The `threadlist` must contain at least one thread (with color) for each color block.
///
/// Readers are not required to provide metadata beyond what is needed for rendering.
use crate::models::EmbPattern;

pub trait EmbroideryReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, Box<dyn std::error::Error>>;
}
