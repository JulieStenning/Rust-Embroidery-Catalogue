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
        self.read_with_report(data).and_then(|report| {
            report.pattern.ok_or_else(|| {
                AppError::parse(report.error.unwrap_or_else(|| "reader returned no pattern".into()))
            })
        })
    }

    fn read_with_report(&self, data: &[u8]) -> Result<ReadReport, AppError> {
        self.read(data).map(ReadReport::success)
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
                    let result = r.read_with_report(&[]);
                    assert!(result.is_ok() || result.is_err());
                    if let Ok(report) = result {
                        assert!(report.pattern.is_some() || report.error.is_some());
                    }
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