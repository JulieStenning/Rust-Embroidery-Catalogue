"""Gemini AI client — low-level request, retry, and response-parsing logic.

This module owns the boundary between the application and the Google Gemini
API.  Higher-level tagging logic lives in ``auto_tagging.py``; this module
only handles:

- constructing and sending API requests
- exponential-backoff retry on transient failures
- stripping markdown fences from JSON responses
- returning raw parsed Python objects to callers
"""

from __future__ import annotations

import json
import logging
import re
import time
from typing import Any

logger = logging.getLogger(__name__)

_DEFAULT_TEXT_MODEL = "models/gemini-2.5-flash-lite"
_DEFAULT_VISION_MODEL = "models/gemini-2.5-flash-lite"


def _strip_fences(text: str) -> str:
    """Remove markdown code fences (```json … ```) if present."""
    text = text.strip()
    if text.startswith("```"):
        text = re.sub(r"^```[a-z]*\n?", "", text)
        text = re.sub(r"\n?```$", "", text)
    return text.strip()


def call_gemini_text(
    prompt: str,
    api_key: str,
    model: str = _DEFAULT_TEXT_MODEL,
    max_retries: int = 3,
    delay_seconds: float = 5.0,
) -> Any:
    """Send a text prompt to Gemini and return the parsed JSON response.

    Args:
        prompt: The full prompt string to send.
        api_key: Google AI API key.
        model: Gemini model identifier.
        max_retries: Number of attempts before giving up.
        delay_seconds: Base delay for exponential backoff.

    Returns:
        Parsed JSON value (dict, list, etc.) from the model response.

    Raises:
        RuntimeError: If all retry attempts fail.
        ValueError: If the response cannot be parsed as JSON.
    """
    from google import genai  # type: ignore[import-untyped]

    client = genai.Client(api_key=api_key)
    last_exc: Exception | None = None

    for attempt in range(1, max_retries + 1):
        try:
            response = client.models.generate_content(model=model, contents=prompt)
            text = _strip_fences(response.text)
            return json.loads(text)
        except Exception as exc:
            last_exc = exc
            wait = delay_seconds * (2 ** (attempt - 1))
            if attempt < max_retries:
                logger.warning(
                    "Gemini text request failed (attempt %d/%d): %s — retrying in %.0fs",
                    attempt,
                    max_retries,
                    exc,
                    wait,
                )
                time.sleep(wait)
            else:
                logger.error(
                    "Gemini text request failed after %d attempts: %s",
                    max_retries,
                    exc,
                )

    raise RuntimeError(f"Gemini text request failed after {max_retries} attempts") from last_exc


def call_gemini_vision(
    image_data: bytes,
    prompt: str,
    api_key: str,
    model: str = _DEFAULT_VISION_MODEL,
    max_retries: int = 3,
    delay_seconds: float = 2.0,
) -> Any:
    """Send a PNG image + prompt to Gemini vision and return the parsed JSON response.

    Args:
        image_data: Raw PNG bytes.
        prompt: Text prompt describing the task.
        api_key: Google AI API key.
        model: Gemini model identifier.
        max_retries: Number of attempts before giving up.
        delay_seconds: Base delay for exponential backoff.

    Returns:
        Parsed JSON value (dict, list, etc.) from the model response.

    Raises:
        RuntimeError: If all retry attempts fail.
        ValueError: If the response cannot be parsed as JSON.
    """
    from google import genai  # type: ignore[import-untyped]
    from google.genai import types  # type: ignore[import-untyped]

    client = genai.Client(api_key=api_key)
    last_exc: Exception | None = None

    for attempt in range(1, max_retries + 1):
        try:
            image_part = types.Part.from_bytes(data=image_data, mime_type="image/png")
            response = client.models.generate_content(
                model=model,
                contents=[image_part, prompt],
            )
            text = _strip_fences(response.text)
            return json.loads(text)
        except Exception as exc:
            last_exc = exc
            wait = delay_seconds * (2 ** (attempt - 1))
            if attempt < max_retries:
                logger.warning(
                    "Gemini vision request failed (attempt %d/%d): %s — retrying in %.0fs",
                    attempt,
                    max_retries,
                    exc,
                    wait,
                )
                time.sleep(wait)
            else:
                logger.error(
                    "Gemini vision request failed after %d attempts: %s",
                    max_retries,
                    exc,
                )

    raise RuntimeError(f"Gemini vision request failed after {max_retries} attempts") from last_exc
