//! Shared helpers for recalculating design technical metadata from the
//! on-disk binary file.
//!
//! The same "re-read the file and re-derive dimensions / recommended hoop"
//! logic is used by the per-design "Recalculate From File" action and the
//! bulk backfill runs from Batch Operations. Keeping it here means a fix or improvement
//! only has to be made once instead of being duplicated in each caller.

use crate::services::image_generation::{generate_preview, ImageGenerationRequest};
use sqlx::SqlitePool;
use std::path::Path;

/// Freshly parsed technical metadata for a design file.  `width_mm` and
/// `height_mm` are already rounded to integer millimetres, matching how the
/// rest of the catalogue stores dimensions.
#[derive(Debug, Clone)]
pub struct ParsedDesignFile {
    pub image_data: Option<Vec<u8>>,
    pub image_type: Option<String>,
    pub width_mm: Option<i64>,
    pub height_mm: Option<i64>,
    pub stitch_count: Option<i64>,
    pub color_count: Option<i64>,
    pub color_change_count: Option<i64>,
}

/// Outcome of a *lenient* parse: the (possibly empty) technical metadata plus
/// an optional error.  A read/decode/extension failure never aborts; instead
/// every metadata field is `None` and `error` carries the reason.
///
/// This is what callers with "flagged import" semantics need — they still want
/// to insert the catalogue row (with `image_data NULL`) so the design surfaces
/// as "needs attention" rather than being silently dropped.
#[derive(Debug, Clone)]
pub struct ParsedDesignFileOutcome {
    pub parsed: ParsedDesignFile,
    pub error: Option<String>,
}

impl ParsedDesignFileOutcome {
    /// True when no preview/metadata could be produced (read or decode failed).
    pub fn is_flagged(&self) -> bool {
        self.error.is_some()
    }
}

/// Read a design file from disk and extract its technical metadata **without
/// failing**.  The same single parsing entry point as [`parse_design_file`],
/// but it returns the (empty) result plus the error instead of short-circuiting
/// to `Err`.  Always renders a 2D preview (no 3D profile).  The original file is
/// never modified.
pub fn parse_design_file_lenient(full_path: &Path) -> ParsedDesignFileOutcome {
    let result = generate_preview(&ImageGenerationRequest {
        file_path: full_path.to_string_lossy().to_string(),
        preview_3d: false,
        preview_3d_profile: None,
    });

    let parsed = ParsedDesignFile {
        image_data: result.image_data,
        image_type: result.image_type,
        width_mm: result.width_mm.map(|value| value.round() as i64),
        height_mm: result.height_mm.map(|value| value.round() as i64),
        stitch_count: result.stitch_count,
        color_count: result.color_count,
        color_change_count: result.color_change_count,
    };

    ParsedDesignFileOutcome {
        parsed,
        error: result.error,
    }
}

/// Read a design file from disk and extract its technical metadata.
///
/// This is the single parsing entry point shared by the "Recalculate From
/// File" action and the bulk backfill actions.  It always renders a 2D preview
/// (no 3D profile) because the callers only need the derived counts/dimensions,
/// not a styled preview.  The original file is never modified.
///
/// Callers that must keep going after a decode failure (flagged import) should
/// use [`parse_design_file_lenient`] instead.
pub fn parse_design_file(full_path: &Path) -> Result<ParsedDesignFile, String> {
    let outcome = parse_design_file_lenient(full_path);
    match outcome.error {
        Some(error) => Err(error),
        None => Ok(outcome.parsed),
    }
}

/// Select the smallest hoop that fits the given design dimensions, trying
/// both orientations.  Mirrors the recommendation logic used during bulk
/// import so recalculated dimensions yield a consistent "Recommended hoop".
pub async fn recommend_hoop_for_design(
    pool: &SqlitePool,
    width_mm: Option<i64>,
    height_mm: Option<i64>,
) -> Result<Option<i64>, String> {
    let (Some(width_mm), Some(height_mm)) = (width_mm, height_mm) else {
        return Ok(None);
    };

    let hoop_id = sqlx::query_scalar::<_, i64>(
        r#"
            SELECT h.id
            FROM hoops h
            WHERE
                (
                    CAST(h.max_width_mm AS REAL) >= CAST(? AS REAL)
                    AND CAST(h.max_height_mm AS REAL) >= CAST(? AS REAL)
                )
                OR (
                    CAST(h.max_width_mm AS REAL) >= CAST(? AS REAL)
                    AND CAST(h.max_height_mm AS REAL) >= CAST(? AS REAL)
                )
            ORDER BY
                (CAST(h.max_width_mm AS REAL) * CAST(h.max_height_mm AS REAL)) ASC,
                CAST(h.max_width_mm AS REAL) ASC,
                CAST(h.max_height_mm AS REAL) ASC,
                h.name COLLATE NOCASE ASC
            LIMIT 1
            "#,
    )
    .bind(width_mm)
    .bind(height_mm)
    .bind(height_mm)
    .bind(width_mm)
    .fetch_optional(pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(hoop_id)
}
