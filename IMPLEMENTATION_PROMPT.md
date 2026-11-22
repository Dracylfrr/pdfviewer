# Lightweight Visual PDF Library — v1.0.0
## Codex Master Implementation Prompt

You are the lead desktop software engineer, Rust engineer, UI/UX engineer, PDF-reader engineer, database engineer, QA engineer, performance engineer, and release engineer for this project.

Build a complete, polished, lightweight, open-source desktop application for organizing and reading personal PDF libraries.

This is intended to become real free software published on GitHub with versioned releases beginning at:

**v1.0.0**

Do not stop at an architecture document or prototype.

Create the actual application, run it, test it, build it, profile it, fix problems, and leave the repository ready for a public GitHub release.

Use a temporary centralized product name if the repository does not already have one.

Make branding easy to rename later from one configuration location.

---

# 1. Product concept

The application combines:

1. A visual PDF library.
2. A folder hierarchy browser.
3. A graph view.
4. A serious PDF reader.
5. Persistent highlights and underlines.
6. Google-Docs-like document notes/comments.
7. Folder-level notes.
8. Bookmarks.
9. Reading themes.
10. Automatic saving.
11. Local search.
12. GitHub-based application updates.

The application should remain simple and lightweight.

It must NOT feel like a large knowledge-management suite filled with unnecessary features.

The primary goal is:

**Find a PDF quickly, understand where it belongs in the library, read it comfortably, annotate it, and never lose those annotations.**

---

# 2. Core example

Suppose a user chooses this library folder:

Books/
    CompSci/
        Algorithms/
            Book1.pdf
            Book2.pdf
        Operating Systems/
            OSBook.pdf
    Mathematics/
        Calculus.pdf

The application must understand the existing physical folder hierarchy.

It should NOT require importing, copying, or moving those PDFs.

The original folders remain the source of truth.

Selecting:

Books

should show:

CompSci
Mathematics

Selecting:

CompSci

should show:

Algorithms
Operating Systems

Selecting:

Algorithms

should show:

Book1.pdf
Book2.pdf

Book1 and Book2 should visually appear together as sibling items underneath the same parent.

---

# 3. Three library navigation modes

Provide three ways to view the exact same library.

## A. Nested Column View

This should be one of the primary navigation methods.

Conceptually similar to a column-based file browser.

Example:

| Books | CompSci | Algorithms | Book |
|---|---|---|---|
| CompSci | Algorithms | Book1.pdf | preview |
| Mathematics | Operating Systems | Book2.pdf | |

When a folder is selected:

its children appear in the next column.

Sibling folders/files remain vertically aligned underneath each other.

This is particularly important.

Do not turn every relationship into a giant graph.

---

## B. Tree View

Provide a conventional collapsible folder tree.

Example:

Books
 ├─ CompSci
 │   ├─ Algorithms
 │   │   ├─ Book1.pdf
 │   │   └─ Book2.pdf
 │   └─ Operating Systems
 └─ Mathematics

Support:

expand
collapse
keyboard navigation
natural sorting
folder-first sorting

---

## C. Graph View

Create an original visual relationship graph inspired conceptually by visual knowledge graphs.

Do NOT copy Anytype's UI, assets, code, or exact design.

Folders and PDFs become nodes.

Relationships include:

folder → child folder

folder → PDF

optional manually-created related-document links

optional tags/collections

---

# 4. Graph behavior

The graph must remain lightweight.

DO NOT render every PDF in a 10,000-document library by default.

Default graph mode should show a scoped neighborhood.

For the selected node, show:

parent
grandparent when useful
siblings
children
manually-related documents

Graph expansion must be progressive.

Provide:

Expand children
Expand parent
Expand related
Collapse branch
Reset view

The user may optionally request:

Show Entire Library

but warn that large libraries may produce a visually dense graph.

---

# 5. Graph layout

Create a clean layout where practical:

ancestors toward the left

selected node near center

children toward the right

siblings arranged vertically

Example:

                Book1.pdf
               /
Books → CompSci → Algorithms
               
                Book2.pdf

Make it obvious that Book1 and Book2 are siblings.

Use distinctive but subtle icons for:

library root
folder
PDF
tag
collection

Folders should use recognizable folder icons.

PDFs should use document icons.

Nodes can display small badges for:

notes
highlights
bookmarks

---

# 6. Lightweight graph requirement

Do not ship an unnecessarily huge graph visualization dependency.

Prefer:

Canvas

SVG

or a small focused graph library.

If a graph library is chosen:

document why it was chosen

measure its bundle impact

do not include a giant dependency merely for basic node rendering.

Only render visible/expanded nodes.

---

# 7. Technology stack

Preferred architecture:

**Tauri 2**
**Rust**
**Svelte**
**TypeScript**
**PDF.js**
**SQLite**

Use Rust for:

filesystem scanning
filesystem watching
database operations
background indexing
file metadata
hashing/fingerprinting
safe native operations

Use Svelte/TypeScript for:

UI
reader controls
annotation interfaces
graph rendering
library views

Use PDF.js for PDF parsing/rendering.

Use SQLite for durable local metadata and annotations.

Do NOT use Electron.

Do NOT embed a Chromium distribution.

Do NOT require:

Node.js at runtime
Python at runtime
Java at runtime
a database server
Docker
a web server
an online account

Development dependencies are acceptable.

---

# 8. Lightweight-first philosophy

Performance is a primary requirement.

Do not optimize only after the app is finished.

Architect for efficiency from the beginning.

The application should:

launch quickly

remain responsive

use little CPU while idle

avoid excessive RAM use

avoid unnecessary disk activity

avoid loading entire libraries into memory

avoid rendering invisible PDF pages

avoid rendering invisible graph nodes

avoid continuously rescanning directories

avoid unnecessary JavaScript dependencies

avoid unnecessary Rust crates

---

# 9. Performance targets

Treat these as goals rather than reasons to compromise correctness:

Cold startup:

preferably within a few seconds on an ordinary modern Windows computer.

Idle CPU:

approximately 0% whenever possible.

Idle GPU:

minimal.

Memory with library open but no large PDF actively rendered:

target well below heavyweight Electron-style applications.

Large library:

must remain usable with thousands of PDFs.

Do not keep every PDF thumbnail, PDF page, graph node, and extracted text document in RAM simultaneously.

---

# 10. Library folder selection

On first launch show:

**Choose Library Folder**

Use the native folder picker.

Allow the user to choose an existing folder containing PDFs and subfolders.

Do not move or copy files.

Store the selected root securely in application settings.

Allow:

Change Library

Add Library

Remove Library From App

Removing a library from the app must NOT delete the folder or PDFs.

---

# 11. Multiple libraries

Support one or more registered library roots.

Example:

Books

University

Research Papers

Manuals

Each library maintains its own hierarchy but uses the same annotation database.

---

# 12. Filesystem safety

By default, the application is a viewer/indexer.

Do not modify users' PDFs.

Do not rename files automatically.

Do not move files automatically.

Do not delete files automatically.

If future file-management actions are implemented, require explicit confirmation.

---

# 13. Filesystem watcher

Watch registered library directories for changes.

Detect:

new PDF
removed PDF
renamed PDF
moved PDF
new folder
removed folder
renamed folder

Debounce filesystem events.

Do not rescan the entire library because one file changed.

Update only the affected subtree when practical.

---

# 14. Incremental indexing

Initial indexing should be asynchronous.

The UI should become usable as quickly as possible.

Show subtle progress such as:

Indexing library…

2,421 files discovered

Do not block application startup.

Index metadata such as:

filename
path
folder
extension
size
modified date
page count when available

Extract heavier information lazily.

---

# 15. Document identity

Annotations must survive reasonable file operations whenever possible.

Do not identify a document only by absolute file path.

Create a stable document identity/fingerprint strategy using inexpensive metadata first.

Avoid calculating the complete cryptographic hash of every multi-hundred-megabyte PDF during every startup.

Use staged identity such as:

stored document ID

path

filename

size

modified time

optional fast fingerprint

and full hash only when needed for ambiguity/recovery.

Document the strategy.

---

# 16. PDF reader layout

When a PDF is opened, use a three-pane layout.

Left:

Library/navigation.

Center:

PDF reader.

Right:

Annotation panel.

Conceptually:

┌───────────────┬────────────────────────────┬────────────────────┐
│ Library       │                            │ Notes              │
│               │                            │                    │
│ Books         │         PDF PAGE           │ Highlight — p. 12  │
│  CompSci      │                            │ My note...         │
│   Algorithms  │                            │                    │
│    Book1.pdf  │                            │ Bookmark — p. 32   │
│    Book2.pdf  │                            │                    │
│               │                            │                    │
└───────────────┴────────────────────────────┴────────────────────┘

Both side panels must be collapsible.

The PDF should be able to occupy almost the entire window when desired.

---

# 17. PDF reader modes

Support:

Continuous scroll

Single page

Page width

Fit page

Custom zoom

Fullscreen reading

Remember the preferred mode.

---

# 18. PDF navigation

Provide:

page number

total pages

previous page

next page

jump to page

document outline/table of contents where available

thumbnail sidebar

search inside document

back/forward navigation history

---

# 19. PDF rendering performance

Do not render every page in a 1,000-page book simultaneously.

Render:

visible pages

plus a small prefetch buffer.

Unload/cull distant pages.

Use PDF.js worker capabilities appropriately.

Cache only reasonable nearby page renders.

Cancel obsolete render requests during fast scrolling.

Avoid duplicate rendering jobs.

Keep scrolling smooth.

---

# 20. Reading progress

Automatically remember:

last page

scroll position

zoom

reader layout

theme

last opened time

per PDF.

Opening a PDF should return the reader to the previous location.

---

# 21. Annotation types

Support these in v1.0:

Highlight

Underline

Bookmark

Document Note

Selection Note

Folder Note

---

# 22. Highlights

The user selects PDF text.

A small floating toolbar appears.

Options:

Highlight

Underline

Add Note

Copy

Allow multiple highlight colors.

Default colors can include:

Yellow

Mint

Blue

Lavender

Pink

Peach

Custom colors may be supported.

---

# 23. Underlining

Allow selected text to be underlined.

Provide several underline colors.

Underlines must:

autosave

remain positioned correctly at different zoom levels

reappear after restarting

work with the right annotation sidebar

---

# 24. Annotation anchoring

Do not store annotations as raw screen pixels only.

Store enough information to reliably relocate them.

Use combinations such as:

document ID

page number

normalized page coordinates

selected text

text position/index where available

annotation type

color

creation time

modified time

note content

Use normalized PDF/page coordinates so zooming does not move annotations incorrectly.

---

# 25. Notes like document comments

The right-hand annotation rail should feel conceptually similar to document comments.

Do NOT copy Google Docs' exact UI.

Each annotation card can display:

type

page number

selected quotation

user note

created date

modified date

color

Clicking a note must jump directly to its PDF location.

Clicking a highlight must select/open the corresponding note card.

---

# 26. Annotation sidebar filters

Right sidebar filters:

All

Notes

Highlights

Underlines

Bookmarks

Allow sorting by:

Page

Newest

Oldest

Color

---

# 27. Folder notes

Folders should also support notes.

Example:

CompSci

Folder Note:

"Core computer-science textbooks and reference material."

When the folder is selected:

the right sidebar can display its folder notes.

Allow multiple folder notes if useful.

Autosave them.

---

# 28. PDF-level notes

Allow notes that are not attached to a text selection.

Example:

"This book is useful mainly for Chapters 4–7."

These belong to the document itself.

---

# 29. Bookmarks

Allow bookmarking individual pages.

Bookmarks may have:

optional title

optional note

color

Example:

Page 83

"Important dynamic programming explanation"

Clicking a bookmark jumps to that page.

---

# 30. Annotation autosave

All changes must autosave.

Use debounced writes.

Example:

user types note

wait approximately 500–1000 ms after typing stops

commit transaction

Show a small state:

Saving…

Saved

Do not show intrusive dialogs.

---

# 31. Reliability priority

Losing highlights or notes is unacceptable.

Use database transactions.

Create backup/recovery support.

Before database schema migrations:

create a backup.

Maintain sensible rotating database backups.

Test sudden termination scenarios where possible.

Never overwrite the only valid copy of annotation data during migration.

---

# 32. SQLite database

Use a local SQLite database.

Suggested data models:

libraries

folders

documents

document_state

annotations

bookmarks

folder_notes

tags

document_tags

relationships

settings

index_metadata

schema_version

Use migrations.

Do not create thousands of separate metadata files unless there is a strong reason.

---

# 33. Database performance

Use indexes for commonly queried fields.

Batch appropriate writes.

Avoid querying the entire annotations table whenever a PDF changes page.

Use prepared statements.

Consider SQLite WAL where appropriate for responsive reads/writes, but implement it correctly and document the decision.

---

# 34. Original PDFs remain unchanged

Annotations should be stored in the application database by default.

Do NOT write annotations into the original PDF.

This protects the user's library.

Provide a later action:

**Export Annotated Copy**

This creates a NEW PDF.

Never overwrite the original unless the user explicitly chooses an overwrite operation.

---

# 35. Export annotations

For v1.0 provide at minimum:

Export annotations as JSON

Export notes as Markdown

If technically reliable within scope, also provide:

Export Annotated PDF Copy

Do not delay core annotation reliability just to implement complex PDF modification.

---

# 36. Annotation import

Allow exported annotation JSON to be imported again.

Validate document identity.

If identity does not match confidently:

ask the user before applying it.

---

# 37. Reading themes

Provide comfortable reading themes.

Preset themes:

Original

Paper White

Warm Cream

Sepia

Charcoal

True Black

Cloud Blue

Lavender

Sage

Mint

Blush Pink

Peach

Soft Gray

Allow:

Custom Background

Custom Text/Ink Color where technically appropriate.

---

# 38. Non-destructive themes

Reading themes must NEVER modify the source PDF.

Themes affect rendering only.

The PDF must appear unchanged when opened in another application.

---

# 39. Theme rendering

Implement reader themes carefully.

For text-heavy PDFs:

allow the page background and ink appearance to adapt to dark/pastel reading themes.

For PDFs containing photographs/illustrations:

avoid destroying image colors when possible.

Provide an option such as:

Preserve Images

If a perfect arbitrary-background transformation is not feasible for every PDF:

implement a robust fallback and clearly document limitations.

Do not fake support by claiming a theme works when it makes PDFs unreadable.

---

# 40. Per-document themes

Allow:

Use Global Theme

or:

Remember Theme For This PDF

Example:

technical book → Warm Cream

novel → Lavender

manual → Dark

---

# 41. Application themes

Separate application chrome theme from PDF page theme.

Application themes:

Light

Dark

System

Pastel Blue

Pastel Lavender

Pastel Sage

Pastel Peach

Keep the UI clean and professional.

Do not make pastel modes childish.

---

# 42. Search

Always provide fast search across:

filename

folder name

tags

document notes

folder notes

bookmarks

annotation notes

highlighted text

---

# 43. Full PDF text indexing

Keep full-library text indexing optional or incremental because the app must remain lightweight.

Default behavior:

extract text from a PDF when opened or when requested.

Provide an optional setting:

**Index PDF text for full-library search**

If enabled:

run indexing at low priority

process incrementally

allow pause

do not freeze the UI

remember completed indexing

do not repeatedly re-index unchanged PDFs.

Do NOT include OCR in v1.0 core.

OCR may be a future optional component.

---

# 44. Search results

Search result cards should show:

document

folder

page when relevant

matched text/snippet

annotation type when relevant

Clicking result opens the exact location where possible.

---

# 45. Tags and collections

Allow optional tags independent of physical folder structure.

Examples:

AI

To Read

Favorite

University

Reference

Research

A PDF remains physically where the user stored it.

Tags create virtual organization only.

---

# 46. Manual relationships

Allow manually connecting two PDFs.

Example:

Computer Architecture

↔

Operating Systems

These relationships can appear in Graph View.

Do not modify filesystem structure.

---

# 47. Relationship types

Keep v1.0 simple.

Support:

Related

Optionally:

Prerequisite

Reference

Continuation

Store relationship type in SQLite.

Do not overcomplicate this into an ontology system.

---

# 48. Library information pane

Selecting a PDF without opening it should show:

title/filename

folder

file size

page count

modified date

tags

notes count

highlights count

bookmarks count

reading progress

Open PDF button

---

# 49. Folder information pane

Selecting a folder should show:

folder name

parent

number of child folders

number of PDFs

folder notes

tags if implemented

recently opened documents inside it

---

# 50. Virtualization

Large lists MUST be virtualized.

Do not create thousands of DOM nodes for:

library lists

search results

annotation lists

thumbnail lists

Only render visible items plus a small buffer.

---

# 51. Thumbnail handling

Generate PDF thumbnails lazily.

Do not create all thumbnails during startup.

Generate when:

visible

requested

or during explicitly enabled idle indexing.

Cache thumbnails on disk.

Use a bounded cache.

Provide cache cleanup.

---

# 52. Memory management

Release PDF documents/pages when no longer needed.

Bound:

render cache

thumbnail cache

graph cache

search result cache

text index memory

Do not allow caches to grow forever.

---

# 53. Background work

Background work must:

be cancellable

yield to user interaction

run at low priority where possible

report progress

never make reading lag because indexing is occurring.

---

# 54. Idle behavior

When the user is simply reading a static page:

CPU usage should become extremely low.

Do not run:

continuous animations

unnecessary polling loops

constant filesystem rescans

constant database queries

constant graph simulation.

If using force-directed graph calculations:

stop the simulation once stable.

---

# 55. Startup behavior

On startup:

load application settings

open database

validate schema

display UI

restore previous library

load visible folder metadata

restore last document if configured

start nonessential work afterward.

Do NOT fully scan and render everything before opening the UI.

---

# 56. Privacy

The application must be local-first.

No account.

No cloud requirement.

No analytics.

No telemetry.

No advertising.

Do not upload:

PDFs

notes

highlights

folder names

search queries

reading history

to any service.

Network access should only occur for:

explicit GitHub update checks

application updates

explicit user-opened links.

Document this in PRIVACY.md.

---

# 57. Update system

Use Tauri's official updater architecture where appropriate.

Support:

Check for Updates

optional automatic startup check

download update

verify update

install update

restart application.

Updates must be cryptographically signed.

Do not create an insecure custom executable updater if Tauri provides the functionality needed.

---

# 58. Update user experience

Settings → Updates:

Current Version

Latest Version

Check for Updates

Automatically Check at Startup

Release Notes

If a new release exists:

show a subtle notification.

Example:

**v1.1.0 is available**

Do not interrupt PDF reading with a forced modal.

Updates must never be mandatory.

---

# 59. Versioning

Use Semantic Versioning.

Initial:

v1.0.0

Examples:

v1.0.1 — bug fix

v1.1.0 — feature release

v2.0.0 — breaking architectural change

Maintain a single canonical version source.

---

# 60. GitHub releases

Create a GitHub Actions release workflow.

On tag:

vX.Y.Z

perform:

tests

lint

production build

Windows packaging

updater signing

checksum generation

GitHub Release creation

release asset upload.

Never commit update private signing keys.

Use GitHub Actions secrets.

---

# 61. Windows release formats

Provide:

normal Windows installer

and, if technically practical:

portable ZIP build.

The application should remain as lightweight as reasonably possible.

Document differences between installer and portable versions.

---

# 62. Open source

Prepare for public GitHub publishing.

Create:

README.md

LICENSE

CHANGELOG.md

ROADMAP.md

CONTRIBUTING.md

SECURITY.md

PRIVACY.md

docs/ARCHITECTURE.md

docs/ANNOTATIONS.md

docs/LIBRARY_INDEXING.md

docs/RELEASING.md

Choose a permissive open-source license compatible with all dependencies unless the repository already specifies one.

Document third-party licenses.

---

# 63. Repository structure

Use a clean structure similar to:

/
README.md
LICENSE
CHANGELOG.md
ROADMAP.md
PRIVACY.md
SECURITY.md
CONTRIBUTING.md

/src
    /components
    /features
        /library
        /graph
        /reader
        /annotations
        /search
        /settings
        /updates
    /stores
    /styles
    /utils

/src-tauri
    /src
        /database
        /filesystem
        /indexing
        /documents
        /updates
        /commands
    /migrations

/tests

/docs

/.github/workflows

Avoid giant files.

Keep functionality modular.

---

# 64. Dependency discipline

Before adding a dependency:

ask whether the standard library or existing dependency can solve the problem.

Avoid redundant UI frameworks.

Avoid giant icon packages if only a small icon subset is needed.

Tree-shake frontend dependencies.

Do not ship testing libraries in production bundles.

Document significant dependency choices.

---

# 65. Reader keyboard shortcuts

Provide sensible shortcuts.

Examples:

Ctrl+O — open PDF

Ctrl+F — search document

Ctrl++ — zoom in

Ctrl+- — zoom out

Ctrl+0 — reset zoom

Ctrl+B — bookmark page

H — highlight selected text when appropriate

Esc — close transient UI

Left/Right or PageUp/PageDown — navigate when appropriate

Make shortcuts discoverable.

---

# 66. Context menus

Right-click PDF:

Open

Open in File Explorer

Copy Path

Add Tag

Add Relationship

Document Info

Right-click folder:

Open

Folder Notes

Open in File Explorer

Copy Path

Do not add dangerous destructive options to v1.0 unnecessarily.

---

# 67. Autosave everywhere

Autosave:

annotation edits

folder notes

document notes

reading position

bookmarks

tags

relationships

theme preferences

layout state.

Autosave should feel invisible.

---

# 68. Backup

Provide:

automatic database backup

manual Backup Library Metadata

Restore Backup

Backup contains app metadata and annotations, not duplicate PDF files.

Make it clear that original PDF files must still be backed up separately by the user.

---

# 69. Error handling

Use calm, understandable errors.

Examples:

"This PDF couldn't be opened."

"The library folder is currently unavailable."

"Your notes were restored from a backup."

"Couldn't check for updates. The app still works normally."

Do not expose stack traces in normal UI.

Log technical details locally.

---

# 70. Missing files

If a PDF disappears:

do NOT immediately erase its annotations.

Mark:

Missing

Keep its metadata and notes.

If the PDF reappears or is relinked:

restore its annotations.

Provide:

Locate File

Forget Missing Document

---

# 71. Corrupted PDFs

A corrupt PDF must not crash the app.

Display:

Unable to render this PDF.

Allow:

Open in default PDF viewer

Open file location

Remove from recent list.

---

# 72. Accessibility

Support:

keyboard navigation

visible focus indicators

UI scaling

adjustable reading zoom

high-contrast option

color-independent annotation indicators

tooltips

reduced animation

screen-reader-friendly labels where practical.

---

# 73. Responsive desktop layouts

The app should work comfortably from approximately:

1024×700 upward.

At smaller widths:

collapse sidebars intelligently.

At larger widths:

make good use of screen space.

Support multi-monitor systems.

Remember window size and position.

---

# 74. No unnecessary animation

This is a productivity/reading application.

Use animation only for:

panel transitions

small loading feedback

graph transitions

subtle hover behavior.

No continuously moving decorative backgrounds.

---

# 75. v1.0 scope

Mandatory for v1.0:

Tauri desktop app

Windows support

select library folder

incremental library indexing

filesystem watching

Nested Column View

Tree View

Graph View

PDF reader

continuous/single page modes

zoom/navigation

PDF text search

highlights

underlines

bookmarks

selection notes

document notes

folder notes

right annotation sidebar

annotation filtering

autosave

SQLite persistence

backup/recovery

reading progress

PDF themes

solid and pastel themes

Gallery-style document information not required

tags

manual related-document links

library metadata search

optional/incremental full-text indexing

GitHub updater

v1.0.0 release packaging

tests

documentation

---

# 76. Do NOT include in v1.0

Avoid scope explosion.

Do not implement unless required by existing repository work:

AI chat

LLM summaries

cloud synchronization

user accounts

collaboration

social features

OCR engine

EPUB reader

DOCX editor

Markdown editor

browser extension

mobile app

remote server

plugin marketplace

complex knowledge ontology

These can be future roadmap items.

---

# 77. Testing

Create automated tests for:

database migrations

annotation persistence

annotation coordinate serialization

highlight recovery

autosave debouncing

bookmarks

folder notes

document notes

library indexing

incremental rescanning

missing files

rename detection

filesystem event handling

search

tags

relationships

theme persistence

reading position

version comparison

updater configuration.

---

# 78. PDF tests

Test PDFs including:

1 page

hundreds of pages

1,000+ pages

text-heavy

image-heavy

mixed text/images

landscape pages

rotated pages

different page sizes

large file size

broken/corrupt file

encrypted/password-protected PDF if supported.

---

# 79. Large-library tests

Generate or mock libraries containing:

100 PDFs

1,000 PDFs

10,000 PDF metadata entries

deeply nested directories

wide directories with thousands of siblings.

Measure:

startup

search

folder switching

graph rendering

RAM

idle CPU.

Do not claim large-library support without testing it.

---

# 80. Performance profiling

Before v1.0 release:

profile startup

profile idle reader

profile scrolling

profile graph

profile indexing

profile search

profile annotation writes.

Look for:

memory leaks

DOM growth

unbounded caches

unnecessary timers

unnecessary database polling

duplicate PDF rendering

filesystem event storms.

Fix meaningful problems before release.

---

# 81. Performance regression documentation

Create:

docs/PERFORMANCE.md

Record representative measurements such as:

startup time

RAM after launch

RAM with medium PDF open

idle CPU

large-library indexing behavior

release package size.

Do not fabricate results.

Record only measured results.

---

# 82. Development phases

Work incrementally.

## Phase 1 — Foundation

Create:

Tauri shell

Svelte UI

theme system

navigation

SQLite database

settings.

Run and test.

---

## Phase 2 — Library

Implement:

folder chooser

recursive indexing

Nested Column View

Tree View

filesystem watching

metadata database.

Test large directory structures.

---

## Phase 3 — PDF Reader

Integrate PDF.js.

Implement:

page rendering

scrolling

zoom

page navigation

search

TOC

reading position.

Profile memory.

---

## Phase 4 — Annotations

Implement:

highlights

underlines

bookmarks

selection notes

document notes

right annotation rail

autosave.

Add recovery tests.

Do not proceed until annotations reliably persist after restart.

---

## Phase 5 — Themes

Implement:

application themes

PDF reading themes

pastel presets

custom colors

per-document theme memory.

Test readability.

---

## Phase 6 — Graph

Implement scoped Graph View.

Do NOT render entire huge libraries by default.

Add progressive expansion and manual relationships.

Profile graph performance.

---

## Phase 7 — Search and organization

Implement:

metadata search

annotation search

tags

optional incremental PDF text indexing.

---

## Phase 8 — Reliability

Implement:

backups

missing-file recovery

relinking

corrupt PDF handling

database migration safety.

---

## Phase 9 — Updater

Implement:

GitHub release integration

Tauri signed updater

manual update check

optional startup check.

Test failure modes.

---

## Phase 10 — Release

Run:

full automated suite

manual QA

performance tests

Windows build

installer build

portable build if practical.

Create v1.0.0 release documentation.

---

# 83. Codex working behavior

Do not implement the entire application in one enormous patch.

Work phase by phase.

After every phase:

run the application

run relevant tests

fix failures

inspect warnings

commit logically if Git access is available.

Do not move forward while a foundational subsystem is obviously broken.

---

# 84. Do not trust assumptions

Inspect actual APIs and dependency versions before implementing against them.

Pin important dependency versions.

Do not invent library methods.

If documentation has changed:

use the current supported API.

---

# 85. No fake functionality

Do not create buttons that pretend to work.

If something is displayed in the UI:

it must work.

Do not leave required v1.0 functionality behind TODO comments.

---

# 86. v1.0 definition of done

Do not call the application v1.0.0 until:

a user can choose a large PDF folder

navigate it with columns/tree/graph

open PDFs

read smoothly

highlight text

underline text

create notes

bookmark pages

close the application

reopen it

see every annotation preserved

navigate from notes to exact pages

change reader background/theme

search library information

add tags/relationships

survive moved/missing PDFs reasonably

check for updates

run fully offline

and build successfully as a Windows release.

---

# 87. Final independent verification

Before completion:

create a fresh test library.

Add nested folders.

Add multiple PDFs.

Launch packaged application.

Register library.

Navigate with all three views.

Open multiple documents.

Add highlights.

Add underlines.

Add document notes.

Add folder notes.

Add bookmarks.

Change themes.

Close application.

Reopen application.

Confirm all annotations remain.

Rename/move a test document.

Confirm recovery behavior.

Temporarily remove a PDF.

Confirm annotations are retained.

Restore it.

Test offline.

Test failed update check.

Test large PDF.

Test large library.

Inspect logs.

Fix reproducible failures.

---

# 88. Final report

When implementation is complete, report:

Application version

Exact technology versions

Implemented features

Automated test count/results

Manual tests performed

Windows build status

Installer size

Portable size if applicable

Measured RAM usage

Measured idle CPU usage

Measured startup time

Large-library test size

Known limitations

Unverified behavior

Exact GitHub v1.0.0 release procedure

Recommended v1.0.1 fixes

Recommended v1.1.0 features.

Do not claim that functionality was tested if it was not actually tested.

---

# Final product principle

The best version of this application is NOT the one with the most features.

The best version is one that:

opens quickly,

shows the user's real folder structure clearly,

makes PDFs pleasant to read,

makes notes and highlights effortless,

never loses annotations,

handles very large libraries gracefully,

uses few computer resources,

works offline,

and remains useful for years.

Prioritize that principle throughout development.