# FONarch — decisions

Locked 2026-09-01 (planning session). Overturns any earlier note that this was an image-filename sorter or a Python+CustomTkinter wrap.

## Product

| Button | Result on Desktop |
|--------|-------------------|
| **GATHER** | Dated folder of renamed, family-sorted fonts (e.g. `FONarch 2026-09-01/`) |
| **ARCHIVE** | Same work, then a zip (`FONarch 2026-09-01.zip`). No leftover folder. |

Each button is a **full run**. ARCHIVE does not require GATHER first. No username, no path, no overwrite quiz. Timestamp the output so a second run the same day doesn’t clobber.

Apache 2.0, not affiliated with Adobe. Personal backup of fonts already on disk — not a redistribution tool. The frozen Python prototype (`python-v1`) stays MIT.

## Name

- Display / wordmark: **FONarch** (FON lockup + arch, spoken like *monarch*)
- GitHub repo: `mickjayofficial/FONarch` (renamed from `FONarchive` 2026-09-05). Old URLs redirect.
- `fonarch.com` is **not a live site, not a FONarch task.**
- A full font manager is a **separate** product. This app stays GATHER / ARCHIVE.

## Stack

Rust + Tauri 2. UI is HTML/CSS. Color themes via CSS variables. Rolling font list is JS fed by Rust events. Left vis is a **WebGL / GLSL** wireframe heightfield (not video, not SVG).

Not Python+PyInstaller, not Electron, not a tray/menu-bar resident. Open-and-run, then quit (or quit from gear / Cmd+Q).

## Hunt (current user only)

1. Known paths: Mac `~/Library/Application Support/Adobe/CoreSync/plugins/livetype/`; Windows `%APPDATA%\Adobe\CoreSync\plugins\livetype\`
2. Adobe-shaped paths under home: `**/Adobe/**/livetype/` with `c` or `.c` `entitlements.xml` (root `<typekitSyncState>`), confirm sibling `r` / `.r` has OpenType
3. Broader home search, skip junk
4. Whole volume last resort, with a status line so it doesn’t look frozen

If two caches match, newest `entitlements.xml`. Never ask which home folder.

**Fingerprint:** `livetype` + entitlements catalog + a font bucket with real OpenType. Numeric IDs. Skip encrypted blobs and `GudeLivetype/` (SQLite).

Mac folders are dotted: `.c/entitlements.xml`, `.r` / `.w` / `.t` (OpenType), `.e` (skip). The 0.9.0 Mac preview only knew this shape.

**Windows (confirmed 2026-09-07):** same known path `%APPDATA%\Adobe\CoreSync\plugins\livetype\`. Folders **drop the dot** and are Hidden: `c` (catalog), `r` / `t` (OpenType), `e` (skip), empty `w` / `u` / `x`, plus `GudeLivetype`. Catalog is `c\entitlements.xml` (`<typekitSyncState>`, child tags). Font files are **hidden** and **extensionless** (`r\10294`, not `.169.otf`); magic is still `OTTO`. Prefer `r` over `w` over `t`. Skip `e`. Hunt/gather accept both Mac dotted and Windows undotted buckets; ids may be extensionless digits. Local/ProgramData livetype copies are not the cache.

**Mac (confirmed 2026-09-03):** catalog child tags under `.c/entitlements.xml`. Real OpenType in `.r` / `.w` / `.t`. Prefer `.r` when an id exists in more than one bucket. Skip `.e` (encrypted blobs) and `GudeLivetype/` (SQLite). Adobe `familyName` splits are kept (Condor / CondorCond / CondorWide; Adorn Banners vs AdornS Banners).

Windows livetype layout is confirmed. Both platform downloads exist (`v1.0.0`). 1.1 is next in a few days.

## Naming fonts

Source of truth is XML **child tags**, not attributes:

```xml
<font>
  <id>169</id>
  <properties>
    <familyName>Proxima Nova</familyName>
    <variationName>Extrabold</variationName>
    <fullName>Proxima Nova Extrabold</fullName>
    <isVariable>false</isVariable>
  </properties>
</font>
```

Path is `livetype/.c/entitlements.xml` on Mac, `livetype/c/entitlements.xml` on Windows — **not** `livetype/entitlements.xml`.

The Python script looked in the wrong place and read attributes, so the 2025-09-19 run had **empty `xml_id` on all 1,391 rows** and guessed with fontTools. That run is a **volume check** (201 families, 1,391 fonts, ~216 MB of fonts), not a naming check.

Fallback: OpenType name table + `fvar`. Trust magic bytes (`OTTO` / `\x00\x01\x00\x00`) over Adobe’s `.otf` extension.

Keep Adobe’s `familyName` splits (CondorWide vs CondorCond, Adorn vs AdornS). Don’t merge them.

Copy with a read/write of the bytes (`copy_font`), not `fs::copy`. On APFS, `fs::copy` clonefiles and keeps Adobe’s mtimes, so a second GATHER looks like a renamed duplicate of the first. Dest files get *now* timestamps. Don’t sleep to fake the roll.

## UI (mockup `mockups/FONarch-UI-v2.png`)

Amber phosphor terminal. Pixel title **FONarch**. Gear (settings) top-left. Custom chrome: minimize + close top-right, plus Cmd+Q / Alt+F4. Left: GLSL wireframe landscape. Right: rolling list. Segmented LED bar. Status line. Two capsule buttons.

Settings fills the glass (titlebar stays). Two columns: THEME | SAVE LOCATION + ABOUT. About: crown (tinted from `--accent`) + **FONarch**, then `VERSION 1.0.0 by Mick Jay` (link https://mdeanjones.net/), `Licensed under Apache 2.0`, `NOT affiliated with Adobe.`, `FONT: Pet Me by Kreative Software`. Dock icon is `assets/identity/FONarch-icon-withBKG.svg` (phosphor `#FFB000` on black). Gear top-left; **1.1** adds a pixel insect beside it (bug report) and Polar donate / key / accent slider in Settings. No crown-on-the-window chrome. No Ko-fi. No Intel Mac. No `fonarch.com`.

### Type

Bundle Kreative’s originals; ship `assets/fonts/FreeLicense.txt`; credit Kreative Software. Do not modify or subset the font files (Relay Fonts 1.2f). `@font-face` the TTFs. Integer-scale the window so 8px cells sit on the pixel grid.

| Face | File | Use |
|------|------|-----|
| **Pet Me 64** | `assets/fonts/PetMe64.ttf` | Title, GATHER / ARCHIVE |
| **Pet Me** | `assets/fonts/PetMe.ttf` | Rolling list, status |

v2 mockup is PETSCII caps on the list. Chrome stays caps. Try mixed-case Pet Me on the live font names (real livetype names are `Proxima Nova Extrabold`); if it kills the terminal read, `text-transform: uppercase` and still store the real names.

### Vis

Locked camera. Displaced grid **undulates in place** (not a flythrough). Full-width mountain range — no center valley/canyon. White wire on black, multiplied by `--accent`. Idle = slow; running = faster; done = back to slow. Heartbeat, **not** a data viz — do not map font count onto mountain height. The LED bar stays the honest progress.

Art direction: Envato Elements loop `assets/video/BG 02.mov` (10 s, 4K ProRes, ~705 MB, white-on-black). **Do not ship or commit that file.** Look-dev accepted 2026-09-02: `mockups/vis-preview.html` (full-width range, idle speed 0.70 / running 2.30). Fine-tune after the Tauri window exists — don’t block on more vis passes.

- Idle: status `Ready...`; list = two-line button instructions; vis slow; bar empty
- Running: live font names, quantized LED from **real file count**, status like `Gathering Minion Pro (12/64)`
- Done: keep the names; status `Saved to Desktop`; do **not** restore help until next launch
- Don’t fake the bar. Family size variation (Noto 72 vs Adorn Serif 1) *is* the organic rhythm
- Integer-scale the window so pixels stay chunky

## License

- Rewrite (this tree): **Apache License 2.0**. Copyright 2026 Mick Jay. `LICENSE` is the official text. About screen: “Apache 2.0. Not affiliated with Adobe.” Apache 2.0 links to https://www.apache.org/licenses/LICENSE-2.0
- Python prototype: **MIT**, frozen on tag `python-v1` / branch `archive/python-v1`. Do not relicense that snapshot.
- Pet Me / Pet Me 64: Kreative Software Relay Fonts Free Use License 1.2f (`assets/fonts/FreeLicense.txt`). About links “Kreative Software” to https://www.kreativekorp.com/

## Python prototype (frozen)

- GitHub tag `python-v1`, branch `archive/python-v1`, release “Python prototype (frozen)”
- Files: `fonarchive_manager.py`, README, LICENSE (MIT)

## Out of scope for this app

Admin elevation, pip, visible `working/`, tray icon, user-swappable Winamp skin engine, scanning other users’ homes, decrypting `.e` blobs.
