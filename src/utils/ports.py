"""
Dynamic localhost port selection for the desktop build.

Used by ``desktop_launcher.py`` to find a free port at runtime so the
installed app starts reliably even when the default development ports
(8002 / 8003) are already occupied on the user's machine.
"""

from __future__ import annotations

import socket
import time
import urllib.error
import urllib.request


def find_free_port(host: str = "127.0.0.1", start: int = 8100, end: int = 9000) -> int:
    """Return a free TCP port on *host* in the range [*start*, *end*).

    Binds a temporary socket with ``SO_REUSEADDR`` disabled and then closes
    it immediately so the chosen port is available for the caller to bind.
    Raises ``OSError`` if no free port is found in the given range.
    """
    for port in range(start, end):
        try:
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
                sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 0)
                sock.bind((host, port))
                return port
        except OSError:
            continue
    raise OSError(f"No free port found on {host} in range {start}–{end}")


def wait_for_server(
    url: str,
    timeout: float = 30.0,
    interval: float = 0.25,
) -> bool:
    """Poll *url* until it returns HTTP 200 or *timeout* seconds elapse.

    Returns ``True`` when the server is ready, ``False`` on timeout.
    Designed to be called with the ``/health`` endpoint so the desktop
    shell only opens the window once the ASGI app is actually serving.
    """
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            with urllib.request.urlopen(url, timeout=2) as resp:
                if resp.status == 200:
                    return True
        except Exception:  # noqa: BLE001 — covers URLError, socket errors, etc.
            pass
        time.sleep(interval)
    return False
