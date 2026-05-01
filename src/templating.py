"""
Shared Jinja2Templates instance with custom filters.
Import `templates` from here in all route modules.
"""

import base64

from fastapi.templating import Jinja2Templates

templates = Jinja2Templates(directory="templates")

# Custom filters
templates.env.filters["b64encode"] = lambda data: (
    base64.b64encode(data).decode("utf-8") if data else ""
)
