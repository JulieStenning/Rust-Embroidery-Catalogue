// SPDX-FileCopyrightText: 2026 Julie Stenning
// SPDX-License-Identifier: GPL-3.0-or-later

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use embroidery_catalogue::models::{EmbPattern, EmbThread, Stitch, StitchType};
use embroidery_catalogue::png_writer::{render_pattern_to_png, RenderSettings};
use embroidery_catalogue::readers::dst_reader::DstReader;
use embroidery_catalogue::readers::embroidery_reader::EmbroideryReader;
use embroidery_catalogue::services::stitch_identifier::suggest_stitching_from_pattern;
use std::collections::HashSet;

/// Helper to generate synthetic patterns with realistic zigzag, satin, and fill runs.
fn generate_synthetic_pattern(num_stitches: usize, num_colors: usize) -> EmbPattern {
    let mut pattern = EmbPattern::new();

    let palette = [
        0x1F77B4, 0xD62728, 0x2CA02C, 0xFF7F0E, 0x9467BD, 0x8C564B, 0xE377C2, 0x17BECF,
    ];
    for i in 0..num_colors {
        pattern
            .threadlist
            .push(EmbThread::new(palette[i % palette.len()]));
    }

    let mut x = 0.0f32;
    let mut y = 0.0f32;
    let color_stride = num_stitches / num_colors.max(1);

    for i in 0..num_stitches {
        if i > 0 && i % color_stride == 0 {
            pattern.stitches.push(Stitch {
                x,
                y,
                stitch_type: StitchType::ColorChange,
            });
            continue;
        }

        // Simulate a dense fill block with alternating steps
        let row = (i / 50) as f32;
        let col = (i % 50) as f32;
        x = col * 4.0;
        y = row * 4.0 + (if i % 2 == 0 { 2.0 } else { -2.0 });

        pattern.stitches.push(Stitch {
            x,
            y,
            stitch_type: StitchType::Stitch,
        });
    }

    pattern
}

/// Helper to encode synthetic stitches into a valid DST binary buffer.
fn generate_synthetic_dst_bytes(num_stitches: usize) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(512 + num_stitches * 3);
    // 512-byte standard DST header
    let header = format!(
        "LA:BENCHMARK DESIGN\rST:{:>7}\rCO:{:>3}\r+X:{:>5}\r-X:{:>5}\r+Y:{:>5}\r-Y:{:>5}\rAX:+    0\rAY:+    0\rMX:+    0\rMY:+    0\rPD:******\r\x1a",
        num_stitches,
        1,
        1000,
        1000,
        1000,
        1000
    );
    bytes.extend_from_slice(header.as_bytes());
    bytes.resize(512, b' ');

    // Simple DST stitch encoding (0 dx, 0 dy regular stitch = 0x03)
    for _ in 0..num_stitches {
        bytes.push(0x00);
        bytes.push(0x00);
        bytes.push(0x03);
    }

    // End record
    bytes.push(0x00);
    bytes.push(0x00);
    bytes.push(0xF3);

    bytes
}

fn bench_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("png_rendering");

    let counts = [100, 2000, 20000];

    for count in counts {
        let pattern = generate_synthetic_pattern(count, 4);

        // 2D Rendering Benchmark
        let settings_2d = RenderSettings::default().with_preview_3d(false);
        group.bench_with_input(
            BenchmarkId::new("render_2d", count),
            &pattern,
            |b, pat| {
                b.iter(|| {
                    let res = render_pattern_to_png(black_box(pat), black_box(&settings_2d));
                    black_box(res).unwrap();
                });
            },
        );

        // 3D Rendering Benchmark
        let settings_3d = RenderSettings::default().with_preview_3d(true);
        group.bench_with_input(
            BenchmarkId::new("render_3d", count),
            &pattern,
            |b, pat| {
                b.iter(|| {
                    let res = render_pattern_to_png(black_box(pat), black_box(&settings_3d));
                    black_box(res).unwrap();
                });
            },
        );
    }

    group.finish();
}

fn bench_stitch_identification(c: &mut Criterion) {
    let mut group = c.benchmark_group("stitch_identification");
    let pattern = generate_synthetic_pattern(15000, 6);
    let mut valid_tags = HashSet::new();
    valid_tags.insert("fill".to_string());
    valid_tags.insert("satin".to_string());
    valid_tags.insert("outline".to_string());
    valid_tags.insert("dense".to_string());

    group.bench_function("identify_15k_stitches", |b| {
        b.iter(|| {
            let tags = suggest_stitching_from_pattern(
                black_box(&pattern),
                black_box("benchmark.dst"),
                black_box("benchmark.dst"),
                black_box(&valid_tags),
                black_box(None),
            );
            black_box(tags);
        });
    });

    group.finish();
}

fn bench_binary_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("binary_parsing");
    let dst_bytes = generate_synthetic_dst_bytes(20000);

    group.bench_function("dst_parse_20k_stitches", |b| {
        b.iter(|| {
            let reader = DstReader;
            let pattern = reader.read(black_box(&dst_bytes)).unwrap();
            black_box(pattern);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_rendering,
    bench_stitch_identification,
    bench_binary_parsing
);
criterion_main!(benches);
