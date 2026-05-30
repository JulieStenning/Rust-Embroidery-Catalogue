# Help and About in the App

This guide explains how to use the in-app Help and About pages.

## Where to Find Help
- Open Help from the top navigation bar.
- Direct route: /help

The Help page is the main in-app guidance hub for everyday tasks.

## What Help Covers
The Help hub currently includes sections for:
- Search
- Importing
- AI Tagging
- Tagging Actions
- Projects
- Maintenance
- Troubleshooting

Use the quick-jump links at the top of the Help page to move directly to each section.

## Where to Find About Documents
- Open About from the footer.
- Direct route: /about

The About page lists bundled project documents that open inside the app.

Current About documents include:
- Disclaimer
- Privacy
- Security
- AI Tagging Guide
- Third-Party Notices
- Licence

If a document is missing from the install, the app returns a not-found page for that document route.

## Workflow Shortcuts to Help
You can open targeted help directly from key screens:
- Browse page: search help link to the Search section.
- Import pages: import help links to Importing section.
- Projects pages: help links to Projects section.
- Orphans maintenance page: links to Maintenance and Troubleshooting sections.

These links are useful when you need guidance without leaving your current task context.

## Troubleshooting Help and About Pages
- If /help does not load, confirm the app is running and refresh.
- If /about loads but a specific document fails, the mapped file may be missing in your installation.
- If a contextual help link lands on the wrong section, use the quick-jump nav on /help and report the broken anchor.

For broader application issues, see [../TROUBLESHOOTING.md](../TROUBLESHOOTING.md).

## Current Limitations
- Help and About are separate pages.
- The Help page does not currently include a dedicated About section.
- About content is limited to bundled project documents mapped by slug.

## Migration Parity Notes (Rust/Svelte)
When validating a rebuilt Help system, keep these behaviors unchanged:
- The same routes must exist: `/help`, `/about`, and `/about/document/{slug}`.
- The Help quick-jump and section labels should remain exactly:
	- `🔍 Search`
	- `📥 Importing`
	- `🤖 AI Tagging`
	- `🏷 Tagging Actions`
	- `📁 Projects`
	- `🛠 Maintenance`
	- `🔧 Troubleshooting`
- Hash links should land on the same section IDs:
	- `#search`
	- `#importing`
	- `#ai-tagging`
	- `#tagging-actions`
	- `#projects`
	- `#maintenance`
	- `#troubleshooting`
- About should still list and open the same document slugs:
	- `disclaimer`
	- `privacy`
	- `security`
	- `ai-tagging`
	- `third-party-notices`
	- `licence`
- Invalid About document slugs, or missing mapped files, should still show a not-found response.

## Related Guides
- [GETTING_STARTED.md](GETTING_STARTED.md)
- [SETTINGS.md](SETTINGS.md)
- [FIRST_IMPORT_ACTIONS.md](FIRST_IMPORT_ACTIONS.md)
- [AI_TAGGING.md](AI_TAGGING.md)
- [BROWSE_BULK_ACTIONS.md](BROWSE_BULK_ACTIONS.md)
- [TAGGING_ACTIONS_BACKFILL.md](TAGGING_ACTIONS_BACKFILL.md)
