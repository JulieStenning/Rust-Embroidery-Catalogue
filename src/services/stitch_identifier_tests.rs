// Tests for the source module.
//
// This module was split out so the production file can stay focused
// on logic. It is included via a #[path] declaration in a
// #[cfg(test)] mod tests; module, retaining full access to the
// private items in the parent module through use super::*;.

use super::*;
use crate::models::{EmbPattern, Stitch};

fn filled_pattern() -> EmbPattern {
    let mut pattern = EmbPattern::new();

    // Dense meandering fill-like block.
    let mut x = 0.0_f32;
    let mut y = 0.0_f32;
    for row in 0..20 {
        for step in 0..40 {
            x = step as f32;
            pattern.stitches.push(Stitch {
                x,
                y,
                stitch_type: StitchType::Stitch,
            });
        }
        y += 0.7;
        pattern.stitches.push(Stitch {
            x,
            y,
            stitch_type: StitchType::Stitch,
        });
        if row % 2 == 0 {
            for step in (0..40).rev() {
                x = step as f32;
                pattern.stitches.push(Stitch {
                    x,
                    y,
                    stitch_type: StitchType::Stitch,
                });
            }
        }
    }

    pattern
}

fn outline_pattern() -> EmbPattern {
    let mut pattern = EmbPattern::new();
    // Sparse perimeter over a large area should score as outline/running.
    for index in 0..80 {
        pattern.stitches.push(Stitch {
            x: index as f32 * 12.0,
            y: 0.0,
            stitch_type: StitchType::Stitch,
        });
    }
    for index in 0..80 {
        pattern.stitches.push(Stitch {
            x: 79.0 * 12.0,
            y: index as f32 * 12.0,
            stitch_type: StitchType::Stitch,
        });
    }
    for index in (0..80).rev() {
        pattern.stitches.push(Stitch {
            x: index as f32 * 12.0,
            y: 79.0 * 12.0,
            stitch_type: StitchType::Stitch,
        });
    }
    for index in (0..80).rev() {
        pattern.stitches.push(Stitch {
            x: 0.0,
            y: index as f32 * 12.0,
            stitch_type: StitchType::Stitch,
        });
    }
    pattern
}

#[test]
fn identifies_filled_for_dense_pattern() {
    let pattern = filled_pattern();
    let valid = HashSet::from([
        "Filled".to_string(),
        "Line Outline".to_string(),
        "Satin Stitch".to_string(),
    ]);

    let tags = suggest_stitching_from_pattern(
        &pattern,
        "dense-fill.pes",
        "C:/imports/filled/dense-fill.pes",
        &valid,
        Some(0.70),
    );

    assert!(tags.contains(&"Filled".to_string()));
}

#[test]
fn identifies_outline_for_sparse_lines() {
    let pattern = outline_pattern();
    let valid = HashSet::from([
        "Filled".to_string(),
        "Line Outline".to_string(),
        "Satin Stitch".to_string(),
    ]);

    let tags = suggest_stitching_from_pattern(
        &pattern,
        "outline.pes",
        "C:/imports/outline/outline.pes",
        &valid,
        Some(0.70),
    );

    assert!(tags.contains(&"Line Outline".to_string()));
}

#[test]
fn identifies_metadata_priority_keyword() {
    // Even if the pattern is empty, name metadata "some_fsl_design.pes" should identify it as Lace.
    let pattern = EmbPattern::new();
    let valid = HashSet::from(["Lace".to_string(), "In The Hoop".to_string()]);

    let tags = suggest_stitching_from_pattern(
        &pattern,
        "some_fsl_design.pes",
        "C:/imports/lace/some_fsl_design.pes",
        &valid,
        Some(0.70),
    );

    assert!(tags.contains(&"Lace".to_string()));
}

#[test]
fn identifies_single_priority_type_for_multi_block_mixed_design() {
    let mut pattern = outline_pattern();
    // Color change
    pattern.stitches.push(Stitch {
        x: 0.0,
        y: 0.0,
        stitch_type: StitchType::ColorChange,
    });
    // Block 2: dense fill
    let mut y = 2000.0_f32;
    for _row in 0..15 {
        for step in 0..20 {
            let x = 2000.0 + step as f32 * 2.0;
            pattern.stitches.push(Stitch {
                x,
                y,
                stitch_type: StitchType::Stitch,
            });
        }
        y += 1.0;
    }

    let valid = HashSet::from(["Filled".to_string(), "Line Outline".to_string()]);

    let tags = suggest_stitching_from_pattern(
        &pattern,
        "mixed.pes",
        "C:/imports/mixed/mixed.pes",
        &valid,
        Some(0.70),
    );

    // Filled has higher priority than outline, so it wins alone.
    assert!(tags.contains(&"Filled".to_string()));
    assert!(!tags.contains(&"Line Outline".to_string()));
}

#[test]
fn identifies_applique_geometric_matching() {
    let mut pattern = EmbPattern::new();
    // Block 1: outline square (placement)
    for i in 0..=5 {
        pattern.stitches.push(Stitch {
            x: i as f32 * 20.0,
            y: 0.0,
            stitch_type: StitchType::Stitch,
        });
    }
    for i in 1..=5 {
        pattern.stitches.push(Stitch {
            x: 100.0,
            y: i as f32 * 20.0,
            stitch_type: StitchType::Stitch,
        });
    }
    for i in 1..=5 {
        pattern.stitches.push(Stitch {
            x: 100.0 - i as f32 * 20.0,
            y: 100.0,
            stitch_type: StitchType::Stitch,
        });
    }
    for i in 1..5 {
        pattern.stitches.push(Stitch {
            x: 0.0,
            y: 100.0 - i as f32 * 20.0,
            stitch_type: StitchType::Stitch,
        });
    }
    pattern.stitches.push(Stitch {
        x: 0.0,
        y: 0.0,
        stitch_type: StitchType::ColorChange,
    });
    // Block 2: identical square (tackdown) but slightly offset
    for i in 0..=5 {
        pattern.stitches.push(Stitch {
            x: i as f32 * 20.0 + 1.0,
            y: 1.0,
            stitch_type: StitchType::Stitch,
        });
    }
    for i in 1..=5 {
        pattern.stitches.push(Stitch {
            x: 101.0,
            y: i as f32 * 20.0 + 1.0,
            stitch_type: StitchType::Stitch,
        });
    }
    for i in 1..=5 {
        pattern.stitches.push(Stitch {
            x: 101.0 - i as f32 * 20.0,
            y: 101.0,
            stitch_type: StitchType::Stitch,
        });
    }
    for i in 1..5 {
        pattern.stitches.push(Stitch {
            x: 1.0,
            y: 101.0 - i as f32 * 20.0,
            stitch_type: StitchType::Stitch,
        });
    }

    let valid = HashSet::from(["Applique".to_string(), "In The Hoop".to_string()]);

    let tags = suggest_stitching_from_pattern(
        &pattern,
        "test_app.pes",
        "C:/imports/test_app.pes",
        &valid,
        Some(0.70),
    );

    // Applique stops the chain - other type tags are not reported.
    assert!(tags.contains(&"Applique".to_string()));
    assert!(!tags.contains(&"In The Hoop".to_string()));
}

/// A filled pattern whose filename contains an "ith" keyword. ITH must win
/// because the priority chain stops at it before filled is considered.
#[test]
fn ith_keyword_beats_filled() {
    let pattern = filled_pattern();
    let valid = HashSet::from(["Filled".to_string(), "In The Hoop".to_string()]);

    let tags = suggest_stitching_from_pattern(
        &pattern,
        "cartoon_ith_toy.pes",
        "C:/imports/ith/cartoon_ith_toy.pes",
        &valid,
        Some(0.70),
    );

    assert!(tags.contains(&"In The Hoop".to_string()));
    assert!(!tags.contains(&"Filled".to_string()));
}

/// A filled pattern whose filename contains an "applique" keyword. Applique
/// must win because the priority chain stops at it before filled is
/// considered.
#[test]
fn applique_keyword_beats_filled() {
    let pattern = filled_pattern();
    let valid = HashSet::from(["Filled".to_string(), "Applique".to_string()]);

    let tags = suggest_stitching_from_pattern(
        &pattern,
        "flower_applique.pes",
        "C:/imports/applique/flower_applique.pes",
        &valid,
        Some(0.70),
    );

    assert!(tags.contains(&"Applique".to_string()));
    assert!(!tags.contains(&"Filled".to_string()));
}

/// A dense uniform fill must be reported as Filled ONLY - not Satin or
/// Outline. This is the core regression for the 53505.hus case.
#[test]
fn filled_suppresses_satin_and_outline() {
    let pattern = filled_pattern();
    let valid = HashSet::from([
        "Filled".to_string(),
        "Line Outline".to_string(),
        "Satin Stitch".to_string(),
    ]);

    let tags = suggest_stitching_from_pattern(
        &pattern,
        "dense-fill.pes",
        "C:/imports/filled/dense-fill.pes",
        &valid,
        Some(0.70),
    );

    assert_eq!(tags, vec!["Filled".to_string()]);
}

/// Verifies the real 53505.hus fixture (a solid dense fill) is reported as
/// Filled ONLY - the original regression that returned filled, outline and
/// satin together. The test is skipped when the fixture file is absent so
/// CI / fresh checkouts do not depend on a local Design folder.
#[test]
fn real_53505_hus_is_filled_only() {
    let path = "target/debug/Data/MachineEmbroideryDesigns/Amazing Designs - Tea Pot Houses Collection I/53505.hus";
    if !std::path::Path::new(path).exists() {
        eprintln!("SKIPPED: fixture file does not exist at {}", path);
        return;
    }

    let valid = HashSet::from([
        "Filled".to_string(),
        "Line Outline".to_string(),
        "Satin Stitch".to_string(),
        "Cross Stitch".to_string(),
    ]);

    let tags = suggest_stitching_from_pattern_file(path, "53505.hus", path, &valid, Some(0.70));

    assert!(tags.contains(&"Filled".to_string()));
    assert!(!tags.contains(&"Line Outline".to_string()));
    assert!(!tags.contains(&"Satin Stitch".to_string()));
    assert!(!tags.contains(&"Cross Stitch".to_string()));
}

/// The other Tea Pot Houses Collection files must also be Filled - not Cross
/// Stitch. Diagonal serpentine fills share the 45/135-degree angle signature
/// with real cross-stitch, so this pins the stitch-length gate that separates
/// them (fills use long rows; cross-stitch legs are short).
#[test]
fn tea_pot_houses_files_are_filled_not_cross_stitch() {
    let base_dir =
        "target/debug/Data/MachineEmbroideryDesigns/Amazing Designs - Tea Pot Houses Collection I";
    let files = [
        "53500.hus",
        "53503.hus",
        "53504.hus",
        "53509.hus",
        "53513.hus",
        "53515.hus",
        "53518.hus",
        "53519.hus",
    ];

    let valid = HashSet::from([
        "Filled".to_string(),
        "Line Outline".to_string(),
        "Satin Stitch".to_string(),
        "Cross Stitch".to_string(),
    ]);

    for file in files {
        let path = format!("{}/{}", base_dir, file);
        if !std::path::Path::new(&path).exists() {
            eprintln!("SKIPPED: fixture file does not exist at {}", path);
            continue;
        }

        let tags = suggest_stitching_from_pattern_file(&path, file, &path, &valid, Some(0.70));
        assert!(
            tags.contains(&"Filled".to_string()),
            "{} should be Filled, got {:?}",
            file,
            tags
        );
        assert!(
            !tags.contains(&"Cross Stitch".to_string()),
            "{} should NOT be Cross Stitch, got {:?}",
            file,
            tags
        );
    }
}

#[test]
fn does_not_identify_applique_for_single_outline() {
    let mut pattern = EmbPattern::new();
    // Block 1: dense fill block
    let mut y = 0.0_f32;
    for _row in 0..15 {
        for step in 0..20 {
            let x = step as f32 * 2.0;
            pattern.stitches.push(Stitch {
                x,
                y,
                stitch_type: StitchType::Stitch,
            });
        }
        y += 1.0;
    }
    // Color change
    pattern.stitches.push(Stitch {
        x: 0.0,
        y: 0.0,
        stitch_type: StitchType::ColorChange,
    });
    // Block 2: outline matching geometrically
    for i in 0..=5 {
        pattern.stitches.push(Stitch {
            x: i as f32 * 7.6,
            y: 0.0,
            stitch_type: StitchType::Stitch,
        });
    }
    for i in 1..=5 {
        pattern.stitches.push(Stitch {
            x: 38.0,
            y: i as f32 * 2.8,
            stitch_type: StitchType::Stitch,
        });
    }
    for i in 1..=5 {
        pattern.stitches.push(Stitch {
            x: 38.0 - i as f32 * 7.6,
            y: 14.0,
            stitch_type: StitchType::Stitch,
        });
    }
    for i in 1..5 {
        pattern.stitches.push(Stitch {
            x: 0.0,
            y: 14.0 - i as f32 * 2.8,
            stitch_type: StitchType::Stitch,
        });
    }

    let valid = HashSet::from(["Applique".to_string(), "In The Hoop".to_string()]);

    let tags = suggest_stitching_from_pattern(
        &pattern,
        "regular.pes",
        "C:/imports/regular/regular.pes",
        &valid,
        Some(0.70),
    );

    assert!(!tags.contains(&"Applique".to_string()));
    assert!(!tags.contains(&"In The Hoop".to_string()));
}
