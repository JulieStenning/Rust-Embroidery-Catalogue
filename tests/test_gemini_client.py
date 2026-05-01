"""Tests for src/services/gemini_client.py — AI integration boundary."""

from __future__ import annotations

import json
from unittest.mock import MagicMock, patch

import pytest

from src.services.gemini_client import _strip_fences, call_gemini_text, call_gemini_vision

# ---------------------------------------------------------------------------
# Shared helpers
# ---------------------------------------------------------------------------


def _mock_genai(response_text: str):
    """Return a sys.modules-ready mock for google.genai and related attrs."""
    mock_response = MagicMock()
    mock_response.text = response_text

    mock_client = MagicMock()
    mock_client.models.generate_content.return_value = mock_response

    mock_genai = MagicMock()
    mock_genai.Client.return_value = mock_client

    mock_types = MagicMock()
    mock_types.Part.from_bytes.return_value = MagicMock()

    return mock_genai, mock_types, mock_client


def _mock_genai_error(exc: Exception):
    """Return a sys.modules-ready mock that always raises exc on API calls."""
    mock_client = MagicMock()
    mock_client.models.generate_content.side_effect = exc

    mock_genai = MagicMock()
    mock_genai.Client.return_value = mock_client

    mock_types = MagicMock()

    return mock_genai, mock_types, mock_client


def _patch_google(mock_genai, mock_types):
    """Context-manager that injects mocks into sys.modules for google.genai."""
    return patch.dict(
        "sys.modules",
        {
            "google": MagicMock(genai=mock_genai),
            "google.genai": mock_genai,
            "google.genai.types": mock_types,
        },
    )


# ---------------------------------------------------------------------------
# _strip_fences helper
# ---------------------------------------------------------------------------


class TestStripFences:
    def test_plain_json_unchanged(self):
        assert _strip_fences('{"a": 1}') == '{"a": 1}'

    def test_strips_triple_backtick_json_lang(self):
        assert _strip_fences('```json\n{"a": 1}\n```') == '{"a": 1}'

    def test_strips_triple_backtick_no_lang(self):
        assert _strip_fences("```\n[1, 2, 3]\n```") == "[1, 2, 3]"

    def test_strips_surrounding_whitespace(self):
        assert _strip_fences("  hello  ") == "hello"

    def test_empty_string(self):
        assert _strip_fences("") == ""


# ---------------------------------------------------------------------------
# call_gemini_text
# ---------------------------------------------------------------------------


class TestCallGeminiText:
    def test_success_returns_parsed_json(self):
        payload = {"flower": ["Flowers"]}
        mock_genai, mock_types, _ = _mock_genai(json.dumps(payload))

        with _patch_google(mock_genai, mock_types):
            result = call_gemini_text("prompt", api_key="key", delay_seconds=0)

        assert result == payload

    def test_markdown_fences_stripped(self):
        payload = {"rose": ["Flowers"]}
        fenced = f"```json\n{json.dumps(payload)}\n```"
        mock_genai, mock_types, _ = _mock_genai(fenced)

        with _patch_google(mock_genai, mock_types):
            result = call_gemini_text("prompt", api_key="key", delay_seconds=0)

        assert result == payload

    def test_missing_api_key_all_retries_fail_raises_runtime_error(self):
        mock_genai, mock_types, _ = _mock_genai_error(Exception("INVALID_API_KEY"))

        with _patch_google(mock_genai, mock_types):
            with patch("src.services.gemini_client.time.sleep"):
                with pytest.raises(RuntimeError, match="failed after"):
                    call_gemini_text("prompt", api_key="bad", max_retries=2, delay_seconds=1)

    def test_transient_error_retried_successfully(self):
        payload = {"cat": ["Cats"]}
        mock_response = MagicMock()
        mock_response.text = json.dumps(payload)

        mock_client = MagicMock()
        mock_client.models.generate_content.side_effect = [
            Exception("transient"),
            mock_response,
        ]
        mock_genai = MagicMock()
        mock_genai.Client.return_value = mock_client
        mock_types = MagicMock()

        with _patch_google(mock_genai, mock_types):
            with patch("src.services.gemini_client.time.sleep"):
                result = call_gemini_text("prompt", api_key="key", max_retries=3, delay_seconds=1)

        assert result == payload

    def test_malformed_json_raises_error(self):
        mock_genai, mock_types, _ = _mock_genai("not valid json !!!!")

        with _patch_google(mock_genai, mock_types):
            with patch("src.services.gemini_client.time.sleep"):
                with pytest.raises((RuntimeError, ValueError, json.JSONDecodeError)):
                    call_gemini_text("prompt", api_key="key", max_retries=1, delay_seconds=0)

    def test_retry_count_respected(self):
        """API is called exactly max_retries times before giving up."""
        mock_genai, mock_types, mock_client = _mock_genai_error(Exception("always fails"))

        with _patch_google(mock_genai, mock_types):
            with patch("src.services.gemini_client.time.sleep"):
                with pytest.raises(RuntimeError):
                    call_gemini_text("prompt", api_key="key", max_retries=3, delay_seconds=1)

        assert mock_client.models.generate_content.call_count == 3

    def test_exponential_backoff_delay_between_retries(self):
        """Test 6.2.5 — Rate-limited requests use exponential backoff delay.

        When the API returns errors, the delay between retries should follow
        ``delay_seconds * 2 ** (attempt - 1)`` (i.e. 1s, 2s, 4s, ...).
        """
        mock_genai, mock_types, mock_client = _mock_genai_error(Exception("rate limited"))

        sleep_calls: list[float] = []

        def track_sleep(seconds: float):
            sleep_calls.append(seconds)

        with _patch_google(mock_genai, mock_types):
            with patch("src.services.gemini_client.time.sleep", side_effect=track_sleep):
                with pytest.raises(RuntimeError):
                    call_gemini_text("prompt", api_key="key", max_retries=3, delay_seconds=1.0)

        # 3 retries → 2 sleep calls between them (attempt 1→2, attempt 2→3)
        assert len(sleep_calls) == 2
        # Exponential backoff: delay * 2^(attempt-1)
        assert sleep_calls[0] == pytest.approx(1.0)  # 1.0 * 2^0 = 1.0
        assert sleep_calls[1] == pytest.approx(2.0)  # 1.0 * 2^1 = 2.0


# ---------------------------------------------------------------------------
# call_gemini_vision
# ---------------------------------------------------------------------------


class TestCallGeminiVision:
    def test_success_returns_parsed_list(self):
        payload = ["Flowers", "Birds"]
        mock_genai, mock_types, _ = _mock_genai(json.dumps(payload))

        with _patch_google(mock_genai, mock_types):
            result = call_gemini_vision(b"\x89PNG", "prompt", api_key="key", delay_seconds=0)

        assert result == payload

    def test_markdown_fences_stripped(self):
        payload = ["Cats"]
        fenced = f"```json\n{json.dumps(payload)}\n```"
        mock_genai, mock_types, _ = _mock_genai(fenced)

        with _patch_google(mock_genai, mock_types):
            result = call_gemini_vision(b"\x89PNG", "prompt", api_key="key", delay_seconds=0)

        assert result == payload

    def test_missing_api_key_all_retries_fail_raises_runtime_error(self):
        mock_genai, mock_types, _ = _mock_genai_error(Exception("INVALID_API_KEY"))

        with _patch_google(mock_genai, mock_types):
            with patch("src.services.gemini_client.time.sleep"):
                with pytest.raises(RuntimeError, match="failed after"):
                    call_gemini_vision(
                        b"\x89PNG", "prompt", api_key="bad", max_retries=2, delay_seconds=1
                    )

    def test_malformed_response_raises_error(self):
        mock_genai, mock_types, _ = _mock_genai("not json at all")

        with _patch_google(mock_genai, mock_types):
            with patch("src.services.gemini_client.time.sleep"):
                with pytest.raises((RuntimeError, ValueError, json.JSONDecodeError)):
                    call_gemini_vision(
                        b"\x89PNG", "prompt", api_key="key", max_retries=1, delay_seconds=0
                    )

    def test_retry_count_respected(self):
        """API is called exactly max_retries times before giving up."""
        mock_genai, mock_types, mock_client = _mock_genai_error(Exception("always fails"))

        with _patch_google(mock_genai, mock_types):
            with patch("src.services.gemini_client.time.sleep"):
                with pytest.raises(RuntimeError):
                    call_gemini_vision(
                        b"\x89PNG", "prompt", api_key="key", max_retries=3, delay_seconds=1
                    )

        assert mock_client.models.generate_content.call_count == 3

    def test_image_bytes_passed_to_api(self):
        """The image data is forwarded to the Gemini parts factory."""
        image_data = b"\x89PNG\r\nfake"
        empty_response: list = []
        mock_genai, mock_types, _ = _mock_genai(json.dumps(empty_response))
        # Ensure the `types` attribute on the genai module resolves to mock_types
        mock_genai.types = mock_types

        with _patch_google(mock_genai, mock_types):
            call_gemini_vision(image_data, "prompt", api_key="key", delay_seconds=0)

        mock_types.Part.from_bytes.assert_called_once_with(data=image_data, mime_type="image/png")
