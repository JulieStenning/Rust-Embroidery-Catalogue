#!/usr/bin/env python3

import argparse
import base64
import io
import json
import sys


def _result(**kwargs):
    payload = {
        "image_base64": None,
        "image_type": None,
        "width_mm": None,
        "height_mm": None,
        "stitch_count": None,
        "color_count": None,
        "color_change_count": None,
        "error": None,
    }
    payload.update(kwargs)
    return payload


def _process_one(file_path, preview_3d, pyembroidery):
    """Process a single file using an already-imported pyembroidery module."""
    try:
        pattern = pyembroidery.read(file_path)
    except Exception as error:  # noqa: BLE001
        return _result(error=f"Could not read embroidery file: {error}")

    if pattern is None:
        return _result(error="Pattern reader returned no data.")

    image_base64 = None
    image_type = "3d" if preview_3d else "2d"
    try:
        buf = io.BytesIO()
        pyembroidery.PngWriter.write(pattern, buf, settings={"guides": False, "fancy": True, "3d": preview_3d})
        image_base64 = base64.b64encode(buf.getvalue()).decode("ascii")
    except Exception as error:  # noqa: BLE001
        image_type = None
        return _result(error=f"Could not render preview image: {error}")

    width_mm = None
    height_mm = None
    try:
        bounds = pattern.bounds()
        if bounds:
            min_x, min_y, max_x, max_y = bounds
            width_mm = round((max_x - min_x) / 10.0, 2)
            height_mm = round((max_y - min_y) / 10.0, 2)
    except Exception:  # noqa: BLE001
        pass

    stitch_count = None
    color_count = None
    color_change_count = None
    try:
        stitch_count = int(pattern.count_stitches())
    except Exception:  # noqa: BLE001
        pass
    try:
        color_count = int(pattern.count_threads())
    except Exception:  # noqa: BLE001
        pass
    try:
        color_change_count = int(pattern.count_color_changes())
    except Exception:  # noqa: BLE001
        pass

    return _result(
        image_base64=image_base64,
        image_type=image_type,
        width_mm=width_mm,
        height_mm=height_mm,
        stitch_count=stitch_count,
        color_count=color_count,
        color_change_count=color_change_count,
    )


def _main() -> int:
    parser = argparse.ArgumentParser(description="Generate image metadata using pyembroidery.")
    parser.add_argument("--file", help="Absolute path to embroidery file (single-file mode)")
    parser.add_argument(
        "--preview-3d",
        default="true",
        choices=["true", "false"],
        help="Whether to render 3D simulation preview",
    )
    parser.add_argument(
        "--batch",
        action="store_true",
        help=(
            "Batch mode: read file paths from stdin (one per line), "
            "output one JSON result per line to stdout. "
            "pyembroidery is imported once and reused for all files."
        ),
    )
    args = parser.parse_args()

    preview_3d = args.preview_3d == "true"

    try:
        import pyembroidery  # pylint: disable=import-error
    except Exception as error:  # noqa: BLE001
        if args.batch:
            for line in sys.stdin:
                file_path = line.rstrip("\n")
                if file_path:
                    print(
                        json.dumps({**_result(error=f"pyembroidery import failed: {error}"), "file_path": file_path}),
                        flush=True,
                    )
        else:
            print(json.dumps(_result(error=f"pyembroidery import failed: {error}")))
        return 0

    if args.batch:
        # Batch mode: pyembroidery imported once above; stream file paths from stdin.
        for line in sys.stdin:
            file_path = line.rstrip("\n")
            if not file_path:
                continue
            result = _process_one(file_path, preview_3d, pyembroidery)
            print(json.dumps({**result, "file_path": file_path}), flush=True)
        return 0

    # Single-file mode (original behaviour).
    if not args.file:
        print(json.dumps(_result(error="--file is required in single-file mode")))
        return 1

    result = _process_one(args.file, preview_3d, pyembroidery)
    print(json.dumps(result))
    return 0


if __name__ == "__main__":
    raise SystemExit(_main())