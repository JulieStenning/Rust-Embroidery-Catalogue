/// Trait for embroidery file readers.
///
/// All implementations **must** populate the returned `EmbPattern` with enough information
/// for rendering a preview image using the PNG renderer. This means:
/// - The `stitches` vector must contain all stitch positions and commands.
/// - The `threadlist` must contain at least one thread (with color) for each color block.
///
/// Readers are not required to provide metadata beyond what is needed for rendering.
use crate::error::AppError;
use crate::models::EmbPattern;

/// Structured result returned by a reader for a single input file.
///
/// The contract is independent of routes and database code so parser failures
/// can be surfaced consistently across preview generation, tagging, and other
/// services.
#[derive(Debug, Clone, Default)]
pub struct ReadReport {
    pub pattern: Option<EmbPattern>,
    pub warnings: Vec<String>,
    pub error: Option<String>,
}

impl ReadReport {
    pub fn success(pattern: EmbPattern) -> Self {
        Self {
            pattern: Some(pattern),
            warnings: Vec::new(),
            error: None,
        }
    }

    pub fn warning(pattern: EmbPattern, warning: impl Into<String>) -> Self {
        Self {
            pattern: Some(pattern),
            warnings: vec![warning.into()],
            error: None,
        }
    }

    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            pattern: None,
            warnings: Vec::new(),
            error: Some(error.into()),
        }
    }
}

pub trait EmbroideryReader {
    fn read(&self, data: &[u8]) -> Result<EmbPattern, AppError> {
        let report = self.read_with_report(data);
        report.pattern.ok_or_else(|| {
            AppError::parse(report.error.unwrap_or_else(|| "reader returned no pattern".into()))
        })
    }

    fn read_with_report(&self, data: &[u8]) -> ReadReport {
        match self.read(data) {
            Ok(pattern) => ReadReport::success(pattern),
            Err(err) => ReadReport::failure(err.to_string()),
        }
    }
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
                    // The only requirement is that the call does NOT panic and returns
                    // a structured report describing the outcome.
                    let report = r.read_with_report(&[]);
                    assert!(report.pattern.is_some() || report.error.is_some());
                }
            }
        };
    }

    /// Generate a dedicated sub-module with mid-stream truncation contract tests.
    ///
    /// Each reader receives a buffer that is valid at the start but is cut off
    /// before the full record set is available. The reader must not panic and
    /// must return a structured report (either a pattern or an error).
    macro_rules! test_reader_truncation {
        ($mod_name:ident, $reader:expr, $data:expr) => {
            mod $mod_name {
                use super::*;

                #[test]
                fn no_panic_on_mid_stream_truncation() {
                    let r = $reader;
                    let report = r.read_with_report($data);
                    assert!(
                        report.pattern.is_some() || report.error.is_some(),
                        "reader must return a pattern or an error for truncated data"
                    );
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

    // Mid-stream truncation buffers: valid header/signature prefix followed by
    // a cut-off record. All must be handled without panicking.
    test_reader_truncation!(
        dst_truncated,
        DstReader,
        // 512-byte header + 2 bytes of a 3-byte stitch record.
        &[0u8; 514][..]
    );
    test_reader_truncation!(
        exp_truncated,
        ExpReader,
        // One full 2-byte stitch + 1 byte of a second stitch.
        &[0x05, 0xF6, 0xFD][..]
    );
    test_reader_truncation!(
        hus_truncated,
        HusReader,
        // Magic + stitch/color counts + partial label.
        &[0u8; 20][..]
    );
    test_reader_truncation!(
        jef_truncated,
        JefReader,
        // Well under the 116-byte fixed header.
        &[0u8; 50][..]
    );
    test_reader_truncation!(
        pes_truncated,
        PesReader,
        // A few header bytes, far short of a full PES header.
        &[0x23, 0x50, 0x45, 0x53, 0x00, 0x00][..]
    );
    test_reader_truncation!(
        vp3_truncated,
        Vp3Reader,
        // Valid signature followed by a cut-off header.
        &b"%vsm%\0\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"[..]
    );
}