"""
Preview and metadata extraction helpers for embroidery files.
"""

import io
import logging
import os
import re

import pyembroidery
from PIL import Image

from src.services.hoops import select_hoop_for_dimensions
from src.services.scanning import ScannedDesign

logger = logging.getLogger(__name__)

# Define preview constants here if not already present
PREVIEW_SCALE = 4.0
PREVIEW_MAX_PX = 512

_FALLBACK_THREAD_PALETTE: list[tuple[int, int, int]] = [
    (220, 20, 60),
    (30, 144, 255),
    (34, 139, 34),
    (255, 140, 0),
    (138, 43, 226),
    (255, 105, 180),
    (0, 206, 209),
    (255, 215, 0),
    (139, 69, 19),
    (70, 70, 70),
]

_MM_DIMS_RE = re.compile(
    r"(\d+(?:[\.,]\d+)?)\s*[x×]\s*(\d+(?:[\.,]\d+)?)\s*mm",
    re.IGNORECASE,
)


def _find_spider_image(filepath: str) -> bytes | None:
    folder = os.path.dirname(filepath)
    basename = os.path.basename(filepath)
    try:
        subdirs = [
            d
            for d in os.listdir(folder)
            if os.path.isdir(os.path.join(folder, d))
            and ("spider" in d.lower() or "embird" in d.lower())
        ]
    except OSError:
        return None
    for subdir in subdirs:
        spider_path = os.path.join(folder, subdir)
        try:
            entries = os.listdir(spider_path)
        except OSError:
            continue
        target = (basename + ".jpeg").lower()
        for entry in entries:
            if entry.lower() == target:
                img_path = os.path.join(spider_path, entry)
                try:
                    with Image.open(img_path) as img:
                        img = img.convert("RGB")
                        buf = io.BytesIO()
                        img.save(buf, format="PNG")
                        return buf.getvalue()
                except Exception:
                    return None
    return None


def _read_spider_text_file(path: str) -> str | None:
    try:
        raw = open(path, "rb").read()
    except OSError:
        return None
    if raw.startswith(b"\xff\xfe"):
        try:
            return raw.decode("utf-16")
        except UnicodeDecodeError:
            return None
    for encoding in ("utf-8", "cp1252", "latin-1"):
        try:
            return raw.decode(encoding)
        except UnicodeDecodeError:
            continue
    return None


def _parse_mm_dimensions(text: str) -> tuple[float, float] | None:
    match = _MM_DIMS_RE.search(text)
    if not match:
        return None
    try:
        width = float(match.group(1).replace(",", "."))
        height = float(match.group(2).replace(",", "."))
    except ValueError:
        return None
    return (round(width, 2), round(height, 2))


def _read_spider_art_dimensions(filepath: str) -> tuple[float, float] | None:
    folder = os.path.dirname(filepath)
    basename = os.path.basename(filepath).lower()
    try:
        subdirs = [
            d
            for d in os.listdir(folder)
            if os.path.isdir(os.path.join(folder, d))
            and ("spider" in d.lower() or "embird" in d.lower())
        ]
    except OSError:
        return None

    for subdir in subdirs:
        spider_path = os.path.join(folder, subdir)
        index_path = os.path.join(spider_path, "index.htm")
        content = _read_spider_text_file(index_path)
        if not content:
            continue

        # Prefer dimensions associated with the exact ART filename.
        name_match = re.search(
            rf"{re.escape(basename)}(.{{0,220}}?){_MM_DIMS_RE.pattern}",
            content,
            flags=re.IGNORECASE | re.DOTALL,
        )
        if name_match:
            try:
                width = float(name_match.group(2).replace(",", "."))
                height = float(name_match.group(3).replace(",", "."))
                return (round(width, 2), round(height, 2))
            except ValueError:
                pass

        # Fallback: look around lines that mention the filename.
        lines = content.splitlines()
        for i, line in enumerate(lines):
            if basename not in line.lower():
                continue
            window = "\n".join(lines[max(0, i - 2) : i + 3])
            dims = _parse_mm_dimensions(window)
            if dims:
                return dims
    return None


def _decode_art_icon(filepath: str) -> bytes | None:
    try:
        import zlib as _zlib

        import compoundfiles  # type: ignore[import]

        def _swizzle(b: int) -> int:
            b ^= 0xD2
            return ((b << 1) | (b >> 7)) & 0xFF

        with open(filepath, "rb") as _f:
            _cf = compoundfiles.CompoundFileReader(_f)
            raw = _cf.open("DESIGN_ICON").read()
        decoded = bytes(_swizzle(b) for b in raw[4:])
        bmp_data = _zlib.decompress(decoded)
        img = Image.open(io.BytesIO(bmp_data)).convert("RGB")
        buf = io.BytesIO()
        img.save(buf, format="PNG")
        return buf.getvalue()
    except Exception:
        return None


def _read_art_metadata(filepath: str) -> dict:
    """Extract Wilcom metadata from a .art file.

    Returns a dict with keys: stitch_count, color_count, vendor.
    All values are None if metadata cannot be read.
    """
    try:
        import struct as _struct

        import compoundfiles  # type: ignore[import]

        with open(filepath, "rb") as _f:
            _cf = compoundfiles.CompoundFileReader(_f)
            metadata_stream = _cf.open("\x05WilcomDesignInformationDDD")
            metadata_blob = metadata_stream.read()

        # Parse Wilcom metadata structure
        if len(metadata_blob) < 160:
            return {"stitch_count": None, "color_count": None, "vendor": None}

        # Read set offset at position 44
        set_off = _struct.unpack_from("<I", metadata_blob, 44)[0]

        # Read stitch_count (at set_off + 44)
        stitch_count = (
            _struct.unpack_from("<i", metadata_blob, set_off + 44)[0]
            if set_off + 44 < len(metadata_blob)
            else None
        )

        # Read color_count (at set_off + 52)
        color_count = (
            _struct.unpack_from("<i", metadata_blob, set_off + 52)[0]
            if set_off + 52 < len(metadata_blob)
            else None
        )

        # Read vendor info (at set_off + 60 for length, set_off + 64 for data)
        vendor = None
        if set_off + 60 < len(metadata_blob):
            vendor_len = _struct.unpack_from("<I", metadata_blob, set_off + 60)[0]
            vendor_start = set_off + 64
            if vendor_start + vendor_len <= len(metadata_blob):
                vendor_bytes = metadata_blob[vendor_start : vendor_start + vendor_len]
                vendor = vendor_bytes.rstrip(b"\x00").decode("utf-8", errors="ignore")

        return {
            "stitch_count": stitch_count,
            "color_count": color_count,
            "vendor": vendor,
        }
    except Exception:
        return {"stitch_count": None, "color_count": None, "vendor": None}


def _thread_color(pattern: pyembroidery.EmbPattern, idx: int) -> tuple[int, int, int]:
    threads = getattr(pattern, "threadlist", None) or []
    try:
        thread = threads[idx]
        color = thread.color
        r = (color >> 16) & 0xFF
        g = (color >> 8) & 0xFF
        b = color & 0xFF
        if r > 230 and g > 230 and b > 230:
            return (200, 200, 200)
        return (r, g, b)
    except (IndexError, AttributeError):
        if idx >= 0:
            return _FALLBACK_THREAD_PALETTE[idx % len(_FALLBACK_THREAD_PALETTE)]
        return (30, 30, 80)


def _vp3_segment_shift_map(
    pattern: pyembroidery.EmbPattern,
    command_mask: int,
) -> dict[int, tuple[float, float]]:
    """Return per-segment XY shifts for VP3 outlier blocks.

    Some VP3 files decode with one large color segment translated from the rest
    of the design. We detect that outlier and align it with the previous large
    segment's centroid.
    """
    segments: list[tuple[float, float, int]] = []
    seg_points: list[tuple[float, float]] = []

    for stitch in pattern.stitches:
        if len(stitch) <= 2:
            continue
        cmd = stitch[2]
        base_cmd = (cmd & command_mask) if isinstance(cmd, int) else cmd
        if base_cmd == pyembroidery.COLOR_CHANGE:
            if seg_points:
                xs = [p[0] for p in seg_points]
                ys = [p[1] for p in seg_points]
                segments.append((sum(xs) / len(xs), sum(ys) / len(ys), len(seg_points)))
                seg_points = []
            continue
        if base_cmd == pyembroidery.STITCH:
            seg_points.append((stitch[0], stitch[1]))

    if seg_points:
        xs = [p[0] for p in seg_points]
        ys = [p[1] for p in seg_points]
        segments.append((sum(xs) / len(xs), sum(ys) / len(ys), len(seg_points)))

    if len(segments) < 3:
        return {}

    large = [seg for seg in segments if seg[2] >= 300]
    if len(large) < 2:
        return {}

    sorted_cy = sorted(seg[1] for seg in large)
    median_cy = sorted_cy[len(sorted_cy) // 2]

    shifts: dict[int, tuple[float, float]] = {}
    for idx in range(1, len(segments)):
        cx, cy, count = segments[idx]
        prev_cx, prev_cy, prev_count = segments[idx - 1]
        if count < 300 or prev_count < 300:
            continue
        if abs(cy - median_cy) <= 140:
            continue
        if abs(prev_cy - median_cy) > 80:
            continue
        if abs(prev_cx - cx) > 120:
            continue
        shifts[idx] = (prev_cx - cx, prev_cy - cy)
    return shifts


def _render_preview(
    pattern: pyembroidery.EmbPattern, ext: str | None = None, preview_3d: bool = True
) -> bytes | None:
    """Render an embroidery pattern preview as PNG bytes.

    Uses pyembroidery's built-in PngWriter. When *preview_3d* is ``True``
    (the default) the render uses 3D stitch simulation for a more realistic
    preview.  When ``False`` a flat 2D render is produced, which is
    significantly faster (2-4x) but less visually detailed.

    The 2D mode is intended for bulk backfill operations where speed
    matters more than visual polish.
    """
    try:
        buf = io.BytesIO()
        settings = {"3d": preview_3d}
        logger.info("PngWriter.write: rendering preview (3d=%s)", preview_3d)
        pyembroidery.PngWriter.write(pattern, buf, settings=settings)
        result = buf.getvalue()
        logger.info("PngWriter.write: succeeded, PNG size=%d bytes", len(result))
        return result
    except Exception as exc:
        logger.error("PngWriter.write: failed: %s", exc, exc_info=True)
        return None


def _process_file(
    filepath: str,
    filename: str,
    rel_filepath: str,
    db,
    generate_preview: bool = True,
    preview_3d: bool = True,
) -> ScannedDesign:

    scanned = ScannedDesign(filename=filename, filepath=rel_filepath)
    ext = os.path.splitext(filename)[1].lower()
    is_art = ext == ".art"
    try:
        spider_dims = _read_spider_art_dimensions(filepath) if is_art else None
        pattern = None
        if is_art:
            # ART support can vary by pyembroidery version; fall back gracefully.
            try:
                pattern = pyembroidery.read(filepath)
            except Exception:
                pattern = None
        else:
            pattern = pyembroidery.read(filepath)

        if pattern is None:
            if spider_dims:
                scanned.width_mm, scanned.height_mm = spider_dims
                hoop = select_hoop_for_dimensions(db, scanned.width_mm, scanned.height_mm)
                if hoop:
                    scanned.hoop_id = hoop.id
                    scanned.hoop_name = hoop.name
            if generate_preview:
                if is_art:
                    scanned.image_data = _find_spider_image(filepath) or _decode_art_icon(filepath)
                else:
                    scanned.image_data = _find_spider_image(filepath)
            return scanned

        # Use shared pattern analysis for bounds, colour counts, and preview
        from src.services.pattern_analysis import analyze_pattern

        analysis = analyze_pattern(
            pattern,
            needs_images=generate_preview,
            needs_color_counts=True,
            preview_3d=preview_3d,
            # Spider dimensions take precedence over pattern bounds for .art files
            existing_width_mm=spider_dims[0] if spider_dims else None,
            existing_height_mm=spider_dims[1] if spider_dims else None,
        )

        # Apply dimensions (spider dims take precedence)
        if spider_dims:
            scanned.width_mm, scanned.height_mm = spider_dims
        else:
            scanned.width_mm = analysis.width_mm
            scanned.height_mm = analysis.height_mm

        # Hoop selection (requires DB — stays here)
        hoop = (
            select_hoop_for_dimensions(db, scanned.width_mm, scanned.height_mm)
            if scanned.width_mm
            else None
        )
        if hoop:
            scanned.hoop_id = hoop.id
            scanned.hoop_name = hoop.name

        # Colour counts
        scanned.stitch_count = analysis.stitch_count
        scanned.color_count = analysis.color_count
        scanned.color_change_count = analysis.color_change_count

        # Preview image — the render already happened inside analyze_pattern
        if generate_preview:
            scanned.image_data = analysis.image_data
            if not scanned.image_data:
                logger.warning(
                    "_process_file: analyze_pattern returned no image for %s, "
                    "falling back to _find_spider_image",
                    filename,
                )
                scanned.image_data = _find_spider_image(filepath)
    except Exception as exc:
        logger.error(
            "_process_file: exception processing %s: %s",
            filename,
            exc,
            exc_info=True,
        )
        scanned.error = str(exc)
    return scanned
