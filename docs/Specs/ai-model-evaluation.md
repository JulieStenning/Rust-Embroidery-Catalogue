# AI Vision Model Evaluation & Selection

## Purpose
This document records the comparative model evaluation and architectural rationale for selecting Google Gemini AI Vision as the provider for automated embroidery design subject classification and tagging.

---

## Context & Evaluation (September 2026)

In September 2026, an evaluation was conducted to benchmark candidate vision models for analysing rendered embroidery stitch preview images (PNG thumbnails) and generating accurate, domain-relevant image tags matching the catalogue taxonomy.

### Models Evaluated
- **Google Gemini AI Vision** (Gemini 1.5 Flash / Flash-Lite / Pro)
- **Anthropic Claude Sonnet** (Claude 3.5 Sonnet Vision)
- **Hugging Face Vision Models** (various open-weights vision-language & image classification models)

---

## Benchmark Findings

1. **Tagging Accuracy:**
   - Interpreting embroidery stitch patterns from rendered 2D line and satin stitch previews is challenging due to stylized textures, non-photorealistic lines, and jump stitch artifacts.
   - While no vision model performed perfectly on 100% of complex designs, Google Gemini AI Vision consistently achieved the highest accuracy in identifying motifs, themes, and subject categories.

2. **Integration & Operational Characteristics:**
   - **Structured Output:** High reliability in producing strict JSON tag list formats required by the background ingestion pipeline.
   - **Throughput & Batching:** Low latency per image and responsive batch concurrency handling.
   - **Cost Effectiveness:** Favorable cost economics for large collections (including a generous free tier for smaller personal catalogues).

---

## Decision & Outcome

> **Outcome:** Gemini AI Vision selected as the primary backend provider due to superior tagging accuracy.

Gemini AI Vision is integrated as an optional, opt-in enrichment step in **Admin → Batch Operations**, complementing the zero-cost local **File & Folder Rules** tagger.
