# Rust-Embroidery-Catalogue

## License
This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version. See the LICENSE file for full details.

## Reader Requirements

All embroidery file readers (DST, PES, JEF, VP3, EXP, etc.) **must** provide enough data for the PNG renderer to generate a preview image. This means:

- The `stitches` vector in `EmbPattern` must contain all stitch positions and commands.
- The `threadlist` must contain at least one thread (with color) for each color block.

Readers are not required to provide metadata beyond what is needed for rendering.

This contract ensures that any supported file can be previewed visually in the catalogue.

---

For more details, see the doc comment on the `EmbroideryReader` trait in `src/readers/embroidery_reader.rs`.
