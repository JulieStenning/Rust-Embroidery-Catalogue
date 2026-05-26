# Batch Tagging Refactor Checklist

Use this checklist when changing settings-driven import tagging behavior or overlap points with unified backfill tagging actions.

## 1. Contract Safety
- [ ] `/import/precheck`, `/import/precheck-action`, `/import/do-confirm`, and `/import/confirm` contracts remain compatible or migration is documented.
- [ ] Any request field changes for import precheck/confirm are backward compatible within the current UI flow.
- [ ] Route-to-service propagation of `run_tier2`, `run_tier3`, and batch settings remains explicit.
- [ ] Line-level references in [docs/Specs/batch-tagging-backend-spec.md](docs/Specs/batch-tagging-backend-spec.md) are updated.

## 2. Settings and Precedence
- [ ] `ai.tier2_auto`, `ai.tier3_auto`, `ai.batch_size`, `ai.delay`, and `import.commit_batch_size` mappings remain correct.
- [ ] API key gating still overrides enabled tier settings when key is missing.
- [ ] Tokenized confirm path and direct confirm path remain behaviorally consistent.
- [ ] Invalid/empty batch settings still coerce safely through parsing/validation paths.

## 3. Tier Semantics Integrity
- [ ] Tier ordering remains Tier1 then Tier2 then Tier3.
- [ ] Tier2 applies only to still-untagged imported designs.
- [ ] Tier3 applies only to still-untagged imported designs with preview image data.
- [ ] `tagging_tier` assignment remains accurate for each applied tier.

## 4. Batch and Commit Semantics
- [ ] `batch_limit` semantics remain an AI-candidate cap (not a unified work-chunk size).
- [ ] Import `commit_batch_size` behavior is validated independently from unified `commit_every`.
- [ ] Any semantic changes are reflected in UI hints and both specs.
- [ ] Distinction between `batch_limit`, unified `batch_size`, and `commit_batch_size` remains documented.

## 5. Overlap Safety With Unified Backfill
- [ ] Shared primitives in `auto_tagging` remain compatible for import and unified backfill callers.
- [ ] Tagging Actions unified trigger path remains stable (`/admin/tagging-actions/run-unified-backfill`).
- [ ] Stop/log endpoints and behavior are unchanged unless intentionally refactored.
- [ ] Cross-reference sections in both specs remain non-contradictory.

## 6. Resilience and Operability
- [ ] Missing API key path remains non-fatal and skips Gemini calls safely.
- [ ] Error paths preserve import completion semantics and avoid partial-state corruption.
- [ ] User-visible precheck warning states (no key/cost notice) remain accurate.
- [ ] Rate-limit guidance remains present when AI features are enabled.

## 7. Test Coverage Gate
- [ ] Updated/added tests in:
  - [tests/test_bulk_import_extra.py](tests/test_bulk_import_extra.py)
  - [tests/test_routes.py](tests/test_routes.py)
  - [tests/test_unified_backfill.py](tests/test_unified_backfill.py)
- [ ] New behavior has at least one route-level and one service-level test.
- [ ] Batch-limit behavior has explicit regression coverage.

## 8. Documentation Gate
- [ ] Current Behavior section updated for implemented behavioral changes.
- [ ] Overlap section with unified backfill remains accurate and minimal.
- [ ] All new critical behavior statements include file+line references.
- [ ] Any externally visible behavior changes are reflected in user-facing docs.
