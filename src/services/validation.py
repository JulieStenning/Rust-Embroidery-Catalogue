"""
Validation helpers shared across service modules.
Mirrors the style of common_validation_functions.py from the old project.
"""

import os


def validate_non_empty_string(value: str, field_name: str) -> str:
    """Strip whitespace; raise ValueError if the result is empty."""
    if not isinstance(value, str):
        raise ValueError(f"{field_name} must be a string.")
    value = value.strip()
    if not value:
        raise ValueError(f"{field_name} must not be empty.")
    return value


def validate_rating(rating: int | None) -> int | None:
    """Rating must be 1–5 or None."""
    if rating is None:
        return None
    if not isinstance(rating, int) or not (1 <= rating <= 5):
        raise ValueError("Rating must be an integer between 1 and 5.")
    return rating


def validate_positive_number(value: float | None, field_name: str) -> float | None:
    """Dimension values must be positive if provided."""
    if value is None:
        return None
    if value <= 0:
        raise ValueError(f"{field_name} must be a positive number.")
    return value


# --- Path validation helpers ---


def validate_path_exists(path: str, field_name: str = "Path") -> str:
    """Raise ValueError if the path does not exist."""
    if not os.path.exists(path):
        raise ValueError(f"{field_name} does not exist: {path}")
    return path


def validate_is_directory(path: str, field_name: str = "Path") -> str:
    """Raise ValueError if the path is not a directory."""
    if not os.path.isdir(path):
        raise ValueError(f"{field_name} is not a directory: {path}")
    return path


def validate_under_base(path: str, base: str, field_name: str = "Path") -> str:
    """Raise ValueError if the path is not under the given base directory."""
    norm_path = os.path.abspath(os.path.normpath(path))
    norm_base = os.path.abspath(os.path.normpath(base))
    if not norm_path.startswith(norm_base):
        raise ValueError(f"{field_name} must be under the managed base folder: {norm_base}")
    return path


def normalize_path(path: str) -> str:
    """Return a normalized absolute path (handles slashes, etc)."""
    return os.path.abspath(os.path.normpath(path))


def user_friendly_path_error(exc: Exception, context: str = "") -> str:
    """Return a user-facing error message for path-related exceptions."""
    msg = str(exc)
    if isinstance(exc, FileNotFoundError):
        return f"File or folder not found{': ' + context if context else ''}."
    if isinstance(exc, PermissionError):
        return f"Permission denied{': ' + context if context else ''}."
    return msg
