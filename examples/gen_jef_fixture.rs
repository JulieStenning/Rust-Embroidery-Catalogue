/// Generate a synthetic JEF fixture file for testing.
///
/// The design spans from (-50, -50) to (750, 250) in deci-mm,
/// giving width = 800 > 700 and height = 300 > 250.
///
/// Usage: cargo run --example gen_jef_fixture
/// Output: tests/Test Designs/SyntheticLarge.jef
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_path = "tests/Test Designs/SyntheticLarge.jef";
    let mut buf: Vec<u8> = Vec::new();

    // ---- 116-byte header -------------------------------------------------
    let stitch_offset: u32 = 116;
    buf.extend_from_slice(&stitch_offset.to_le_bytes()); // 0..4
    buf.extend_from_slice(&[0u8; 20]); // 4..24  _pad1
    buf.extend_from_slice(&0u32.to_le_bytes()); // 24..28 count_colors = 0
    buf.extend_from_slice(&[0u8; 88]); // 28..116 _pad2

    // ---- Stitch deltas --------------------------------------------------
    // Each pair is (dx, dy) in signed bytes.
    // For dy, the parser does: result_y = -(signed8(byte))
    //   so to produce result_y = -50 we store byte = 50
    //   and to produce result_y =  50 we store byte = 206
    //
    //   For dx the parser does: result_x = signed8(byte) (no negation).

    let deltas: &[(i16, i16)] = &[
        // Move from origin to (-50, -50)
        (-50, -50),
        // Move right in 100-step increments → max x = 750
        (100, 0),
        (100, 0),
        (100, 0),
        (100, 0),
        (100, 0),
        (100, 0),
        (100, 0),
        (100, 0), // 8 × 100 = 800, but first was -50, so max_x = -50+800 = 750
        // Move down in 50-step increments → max y = 250
        (0, 50),
        (0, 50),
        (0, 50),
        (0, 50),
        (0, 50),
        (0, 50), // 6 × 50 = 300, first was -50, so max_y = -50+300 = 250
    ];

    for &(dx, dy) in deltas {
        // dx byte: store the signed byte directly (parsed as signed8(byte))
        let dx_byte = dx as u8;
        // dy byte: parser does -(signed8(byte)), so we store the negation
        // of the desired dy.  If desired dy is -50, we store 50.
        // If desired dy is  50, we store 206.
        let dy_byte = (-dy) as u8;
        buf.push(dx_byte);
        buf.push(dy_byte);
    }

    let mut file = std::fs::File::create(out_path)?;
    file.write_all(&buf)?;
    println!("Wrote {} bytes to {}", buf.len(), out_path);
    println!(
        "Stitches: {}, bounding box expected: width=800 height=300",
        deltas.len()
    );
    Ok(())
}
