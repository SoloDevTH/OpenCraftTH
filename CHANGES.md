# What OpenCraft Studio changed

OpenCraft Studio starts from a snapshot (October 2026) of the `main` branch of each original app at
[github.com/storytold](https://github.com/storytold), by the ArtCraft team. Everything below was
changed in this fork; the rest of each app is as its authors wrote it. The original files are under
MIT OR Apache-2.0, and so are these changes.

## Name, links and installers

- **No ArtCraft name or logos.** The originals' brand terms (`docs/brand/LICENSE-brand.txt`) ask forks
  and modified versions to remove them, so:
  - every `docs/brand/` folder (the ArtCraft wordmark and mark) is gone, with its attribution entries;
  - the apps that showed the ArtCraft mark now show their own app icon: EffectCraft (header and
    About), FilmCraft (About and start screen), PdfCraft (About and Home), DesignCraft (About and
    start screen);
  - "ArtCraft" in the apps' interface text, in every language, now reads "OpenCraft Studio";
  - the READMEs open with a notice that says what this fork is and whose work it is based on;
    "By the ArtCraft team" and the copyright lines are kept, as credit;
  - one README screenshot that showed the ArtCraft mark (PdfCraft's tool catalogue) was removed.
- **Links point at this repository.** Help menus, About boxes, start screens, CLI help and the
  installers link to `github.com/SoloDevTH/OpenCraftTH` (the app's folder, Issues,
  Releases) instead of getartcraft.com and the original repositories. Links that credit others
  (contributors, fonts, models, craft-libs) are unchanged.
- **Publisher.** Installers, Windows file properties and Linux packages name the fork's owner
  instead of the original publisher.
- **New installer identities.** Every MSI has a new UpgradeCode and new component GUIDs, so
  installing OpenCraft Studio never upgrades, replaces or removes an official install of an
  original app.

## Windows integration

- **Open with and Default apps for every app** (`<app>/packaging/windows/<app>.wxs`): each MSI
  registers a ProgId with its icon, `OpenWithProgids` for every file type the app opens,
  `Applications\<exe>\SupportedTypes`, and `Capabilities` + `RegisteredApplications`, so the app
  shows under **Open with** and in **Settings ▸ Apps ▸ Default apps**. An app is made the default
  only for its own document format; shared formats are offered, never taken over.
- **One-file installer** (`installer/`): a WiX Burn bundle that chains the ten MSIs, with a page of
  checkboxes to choose programs (each with a line in English and in Thai on what it does), and a
  Modify page to add or remove programs later. `installer/build.cmd` builds everything.
  Its version adds the minute of the day, so two builds on the same day still upgrade.

## Built in instead of downloaded

- **LightCraft place names**: the GeoNames data (cities of 15 000+ people, regions, countries;
  CC BY 4.0) ships in `geonames\` beside the program, so reverse geocoding works offline from the
  first start (`crates/engine/src/cmd/geocode.rs`; *Download Place Names* still updates a copy in
  the user's settings, which then wins).
- **EffectCraft models**: the Roto Brush model (MobileSAM, Apache-2.0) and the face-tracking model
  (MediaPipe Face Landmarker, Apache-2.0) ship in `models\` beside the program, each with its
  NOTICE (`crates/engine/src/models.rs`). Settings shows them as *Built in*; they can't be removed.

## Removed

- **Discord**: every Discord button, link, menu item and `help.discord` command, in all apps,
  languages, CLIs, MCP descriptions, installers and READMEs.
- **PdfCraft's update check request**: *Help ▸ Check for updates* opens the Releases page instead of
  asking the GitHub API, because this fork's releases are tagged by date, not by PdfCraft's
  version. PdfCraft no longer depends on `ureq` and `rustls-native-certs`.

## Fixes

- GridCraft: a `clippy::let_unit_value` error on Windows (`crates/ui-egui/src/desktop_theme.rs`).
- PdfCraft: `packaging/windows/test-msi.ps1` failed under Windows PowerShell 5.1 (a non-ASCII
  character read in the ANSI code page), so the portable zip was never made.
- PdfCraft: the OCR test now scans at 300 dpi (at 150 it read "quick" as "auick" on x86-64) and
  checks "same line" relative to word height.
- LightCraft: four tests that failed only on a busy machine (fixed 3-20 s waits now 120 s like the
  rest of the suite; a stage-cache budget test that left less headroom than the caches' own
  two-render capacity).

Generated files refreshed to match: PdfCraft's `ATTRIBUTION.toml` hashes of the translation
catalogs edited above (and `ATTRIBUTION.md` from it), and PhotoCraft's `docs/scorecard.md`.

## Earlier local changes in this snapshot

The working copy this fork started from already had local changes, which are kept:

- DesignCraft, FilmCraft, LightCraft: the loopback control channel requires a bearer token
  (`crates/engine/src/control_auth.rs`), and FilmCraft and LightCraft limit the files automation
  may read and write to roots given at launch (`crates/engine/src/automation_roots.rs`), as
  PhotoCraft already did.
- Smaller edits across the UI crates. For the complete difference, compare an app folder with its
  original repository.

## Not changed

- App names (PdfCraft, PhotoCraft, …), their app icons, and their internal identifiers
  (`ai.storyteller.<app>` for Linux desktop files and macOS bundles, ProgIds such as
  `PdfCraft.Document`).
- Developer notes (`AGENTS.md`, `CLAUDE.md`, `docs/`, roadmaps) describe the original projects and
  were left as written, apart from removed Discord links.
- Each app's `LICENSE-MIT`, `LICENSE-APACHE` and `NOTICE`.
