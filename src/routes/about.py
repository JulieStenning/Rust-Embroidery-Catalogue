"""About page with links to user-facing project documents."""

from __future__ import annotations

from pathlib import Path

from fastapi import APIRouter, HTTPException, Request
from fastapi.responses import HTMLResponse

from src.templating import templates

router = APIRouter(tags=["about"])

_PROJECT_ROOT = Path(__file__).resolve().parents[2]
_DOCUMENTS = {
    "disclaimer": {
        "title": "Disclaimer",
        "filename": "DISCLAIMER.html",
        "description": "Important use-at-your-own-risk and limitation-of-liability information.",
    },
    "privacy": {
        "title": "Privacy",
        "filename": "PRIVACY.md",
        "description": "Explains what data is stored locally and what optional AI features may send externally.",
    },
    "security": {
        "title": "Security",
        "filename": "SECURITY.md",
        "description": "Guidance on secrets, API keys, portable deployments, and safe usage.",
    },
    "ai-tagging": {
        "title": "AI Tagging Guide",
        "filename": "docs/AI_TAGGING.md",
        "description": "How to get a Google API key, enable optional AI tagging, and understand likely usage costs.",
    },
    "third-party-notices": {
        "title": "Third-Party Notices",
        "filename": "THIRD_PARTY_NOTICES.md",
        "description": "Licensing and attribution information for bundled and dependency software.",
    },
    "licence": {
        "title": "Licence",
        "filename": "LICENSE",
        "description": "The licence terms for the Embroidery Catalogue project itself.",
    },
}


@router.get("/about", response_class=HTMLResponse)
def about_page(request: Request):
    documents = []
    for slug, doc in _DOCUMENTS.items():
        path = _PROJECT_ROOT / doc["filename"]
        documents.append(
            {
                **doc,
                "slug": slug,
                "available": path.exists(),
            }
        )

    return templates.TemplateResponse(
        request,
        "about.html",
        {
            "documents": documents,
        },
    )


@router.get("/about/document/{slug}", response_class=HTMLResponse)
def about_document_page(request: Request, slug: str):
    doc = _DOCUMENTS.get(slug)
    if doc is None:
        raise HTTPException(status_code=404, detail="Document not found.")

    path = _PROJECT_ROOT / doc["filename"]
    if not path.exists():
        raise HTTPException(status_code=404, detail="Document file is missing.")

    return templates.TemplateResponse(
        request,
        "about_document.html",
        {
            "title": doc["title"],
            "filename": doc["filename"],
            "document_text": path.read_text(encoding="utf-8", errors="replace"),
        },
    )
