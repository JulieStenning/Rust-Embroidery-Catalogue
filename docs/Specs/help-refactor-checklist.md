# Help Refactor Checklist

Use this checklist when changing Help routes, About routes, help template content, or contextual help links across templates.

## 1. Contract Safety
- [ ] /help route path and method remain compatible, or migration is documented.
- [ ] /about and /about/document/{slug} contracts remain compatible, or migration is documented.
- [ ] Slug-based document lookup behavior remains explicit and safe for missing/unknown slugs.
- [ ] References in [docs/Specs/help-backend-spec.md](docs/Specs/help-backend-spec.md) are updated.

## 2. Navigation and Discoverability
- [ ] Global Help discoverability remains present in [templates/base.html](templates/base.html).
- [ ] About discoverability remains present in footer/navigation surfaces.
- [ ] Contextual help links remain available from browse, import, projects, and maintenance workflows.
- [ ] Any link target changes are reflected in help sections and tests.

## 3. Help Content Integrity
- [ ] Help section anchors remain stable or redirects/migrations are documented.
- [ ] Help content wording remains consistent with active UI labels and workflow steps.
- [ ] Troubleshooting guidance remains aligned with supported user paths.
- [ ] Any removed sections are replaced with clear alternatives.

## 4. About Document Rendering Safety
- [ ] About document slug map remains explicit and reviewed.
- [ ] Missing document files continue to fail safely with 404 behavior.
- [ ] New document links are validated against actual bundled files.
- [ ] About document template still renders plain, readable content for markdown/html sources.

## 5. Contextual Link Consistency
- [ ] Browse page help links still point to the intended help anchors.
- [ ] Import workflow links still point to importing/troubleshooting help sections.
- [ ] Projects pages still expose projects help guidance.
- [ ] Maintenance/orphans pages still expose maintenance and troubleshooting help guidance.

## 6. Known Gaps Governance
- [ ] Known gaps section in the help spec is reviewed when behavior changes.
- [ ] New constraints are documented explicitly rather than implied.
- [ ] Closed gaps are removed from known-gaps list and reflected in target architecture.
- [ ] Future direction statements are updated when roadmap priorities change.

## 7. Test Coverage Gate
- [ ] Updated/added route tests in [tests/test_routes.py](tests/test_routes.py) for /help and /about behavior changes.
- [ ] Help heading/anchor expectations remain covered by tests.
- [ ] About document route behavior (valid slug, invalid slug, missing file) remains covered.
- [ ] Contextual-link regressions have at least one route/template-level assertion when changed.

## 8. Documentation Gate
- [ ] [docs/Specs/help-backend-spec.md](docs/Specs/help-backend-spec.md) reflects current behavior.
- [ ] [docs/User-Facing-Guidance/HELP.md](docs/User-Facing-Guidance/HELP.md) remains aligned with in-app help navigation.
- [ ] [docs/feature-inventory.md](docs/feature-inventory.md) links to current help spec/checklist/user guidance.
- [ ] Changelog entry is added when user-visible help behavior changes.
