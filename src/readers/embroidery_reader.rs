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

// ---------------------------------------------------------------------------
// Trait conformance tests — verify every reader implements the contract
// without panicking on empty/invalid input.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod reader_conformance_tests {
    use super::EmbroideryReader;
    use crate::readers::{DstReader, ExpReader, HusReader, JefReader, PesReader, Vp3Reader};

    /// Generate a dedicated sub-module with contract tests for a single reader.
    macro_rules! test_reader_contract {
        ($mod_name:ident, $reader:expr) => {
            mod $mod_name {
                use super::*;

                #[test]
                fn no_panic_on_empty_data() {
                    let r = $reader;
                    // The trait does not mandate Ok for empty input — some readers
                    // return Err if the header signature or minimum bytes are missing.
                    // The only requirement is that the call does NOT panic.
                    let _result = r.read(&[]);
                }
            }
        };
    }

    test_reader_contract!(dst, DstReader);
    test_reader_contract!(exp, ExpReader);
    test_reader_contract!(hus, HusReader);
    test_reader_contract!(jef, JefReader);
    test_reader_contract!(pes, PesReader);
    test_reader_contract!(vp3, Vp3Reader);
}