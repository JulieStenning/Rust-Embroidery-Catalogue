#!/usr/bin/env python3

import argparse
import base64
import io
import json


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


def _main() -> int:
    parser = argparse.ArgumentParser(description="Generate image metadata using pyembroidery.")
    parser.add_argument("--file", required=True, help="Absolute path to embroidery file")
    parser.add_argument(
        "--preview-3d",
        default="true",
        choices=["true", "false"],
        help="Whether to render 3D simulation preview",
    )
    args = parser.parse_args()

    preview_3d = args.preview_3d == "true"

    try:
        import pyembroidery  # pylint: disable=import-error
    except Exception as error:  # noqa: BLE001
        print(json.dumps(_result(error=f"pyembroidery import failed: {error}")))
        return 0

    try:
        pattern = pyembroidery.read(args.file)
    except Exception as error:  # noqa: BLE001
        print(json.dumps(_result(error=f"Could not read embroidery file: {error}")))
        return 0

    if pattern is None:
        print(json.dumps(_result(error="Pattern reader returned no data.")))
        return 0

    image_base64 = None
    image_type = "3d" if preview_3d else "2d"
    try:
        buf = io.BytesIO()
        pyembroidery.PngWriter.write(pattern, buf, settings={"guides": False, "fancy": True, "3d": preview_3d})
        image_base64 = base64.b64encode(buf.getvalue()).decode("ascii")
    except Exception as error:  # noqa: BLE001
        image_type = None
        print(json.dumps(_result(error=f"Could not render preview image: {error}")))
        return 0

    width_mm = None
    height_mm = None
    try:
        bounds = pattern.bounds()
        if bounds:
            min_x, min_y, max_x, max_y = bounds
            width_mm = round((max_x - min_x) / 10.0, 2)
            height_mm = round((max_y - min_y) / 10.0, 2)
    except Exception:  # noqa: BLE001
        width_mm = None
        height_mm = None

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

    print(
        json.dumps(
            _result(
                image_base64=image_base64,
                image_type=image_type,
                width_mm=width_mm,
                height_mm=height_mm,
                stitch_count=stitch_count,
                color_count=color_count,
                color_change_count=color_change_count,
            )
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(_main())