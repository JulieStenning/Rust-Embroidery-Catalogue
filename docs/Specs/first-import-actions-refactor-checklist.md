# First Import Actions Refactor Checklist

Use this checklist when changing first import actions behavior in the import wizard and import-mode admin review pages.

## 1. Contract Safety
- [ ] Endpoint methods and paths remain compatible for /import/precheck, /import/precheck-action, and /import/do-confirm.
- [ ] Action values accepted by precheck-action remain stable or have an explicit migration plan.
- [ ] Import-mode admin list routes preserve token-aware behavior.
- [ ] Evidence links in [docs/Specs/first-import-actions-backend-spec.md](docs/Specs/first-import-actions-backend-spec.md) are updated for moved symbols.

## 2. First Import Actions Semantics
- [ ] First import detection remains based on catalogue design count.
- [ ] Subsequent import behavior continues offering optional review paths.
- [ ] First import still emphasizes hoops setup before import.
- [ ] Skip-hoops confirmation remains explicit before first-import import_now execution.

## 3. Multi-Entity Review Coverage
- [ ] Precheck page continues exposing review actions for hoops, tags, sources, and designers.
- [ ] Import-mode banners remain present on all four admin pages when token is valid.
- [ ] Cross-links among review pages remain available in import mode.
- [ ] Continue-with-import controls remain available from each review page.

## 4. Token Integrity and Lifecycle
- [ ] Token validation remains strict UUIDv4 format checking.
- [ ] Unknown or invalid tokens continue to fail safe with redirect to /import/.
- [ ] Confirm path remains token-consuming and single-use.
- [ ] Any lifecycle changes (TTL, cleanup, cancellation invalidation) are documented in the backend spec.

## 5. Import Runtime Integration
- [ ] Saved settings still map correctly into precheck and confirm execution path.
- [ ] API key gating still enforces Tier 1-only behavior when key is absent.
- [ ] Image preference override from precheck still propagates into confirm runtime behavior.
- [ ] Commit batch settings remain consistent with service defaults or are explicitly documented.

## 6. Resilience and Operability
- [ ] No selected files path still redirects safely to /import/.
- [ ] Cancel actions still return to /import/ without import side effects.
- [ ] UI continue actions keep loading-overlay behavior on import-mode pages.
- [ ] Error and edge paths preserve predictable user recovery.

## 7. Test Coverage Gate
- [ ] Updated or added route tests in [tests/test_routes.py](tests/test_routes.py).
- [ ] Updated or added first-vs-subsequent behavior tests in [tests/test_bulk_import_extra.py](tests/test_bulk_import_extra.py).
- [ ] Updated or added end-to-end regression anchors in [tests/test_regression_e2e.py](tests/test_regression_e2e.py) when cross-page flow changes.
- [ ] Token safety changes include both valid-token and invalid-token regression coverage.

## 8. Documentation Gate
- [ ] [docs/Specs/first-import-actions-backend-spec.md](docs/Specs/first-import-actions-backend-spec.md) is updated with behavior and line-level references.
- [ ] [docs/User-Facing-Guidance/FIRST_IMPORT_ACTIONS.md](docs/User-Facing-Guidance/FIRST_IMPORT_ACTIONS.md) remains aligned with current behavior.
- [ ] Feature is consistently framed as first import actions, not tag-review-only.
- [ ] Any future-facing notes are explicitly marked as target architecture, not current behavior.
