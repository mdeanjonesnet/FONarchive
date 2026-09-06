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
- `fonarch.com` is a Namecheap squat (Jul 2026, WHOIS never verified). `.app` / `.dev` / `.io` looked free as of 2026-09-01
- Future “FontBase on steroids” AI manager is a **separate** product and repo

## Stack

Rust + Tauri 2. UI is HTML/CSS. Color themes via CSS variables. Rolling font list is JS fed by Rust events. Left vis is a **WebGL / GLSL** wireframe heightfield (not video, not SVG).

Not Python+PyInstaller, not Electron, not a tray/menu-bar resident. Open-and-run, then quit (or quit from gear / Cmd+Q).

## Hunt (current user only)

1. Known paths: Mac `~/Library/Application Support/Adobe/CoreSync/plugins/livetype/`; Windows `%APPDATA%\Adobe\CoreSync\plugins\livetype\`
2. Adobe-shaped paths under home: `**/Adobe/**/livetype/.c/entitlements.xml` with root `<typekitSyncState>`, confirm sibling `.r` has OpenType
3. Broader home search, skip junk
4. Whole volume last resort, with a status line so it doesn’t look frozen

If two caches match, newest `entitlements.xml`. Never ask which home folder.

**Fingerprint:** `livetype` + `.c/entitlements.xml` + hidden `.r` / `.e` / `.w` (and `.t`). Numeric IDs.

**This Mac (2026-09-03 GATHER):** catalog **1,822** fonts (1,382 `OS` + 440 `CC`) / **263** families, 10 variable. Real OpenType: `.r` 1,382, `.w` 381, `.t` 60 (one id in both `.r` and `.w` — prefer `.r`). Skip `.e` (1,822 encrypted blobs) and `GudeLivetype/` (SQLite). First Rust run: `~/Desktop/FONarch 2026-09-03/` (1,822 files, 270 MB, 1,344 OTF + 478 TTF). Adobe `familyName` splits are kept (Condor / CondorCond / CondorWide; Adorn Banners vs AdornS Banners).

**Windows test (after Mac 0.9):** nearer candidate is Michael’s Win 11 gaming laptop if Saturday-morning Adobe CC + fonts produce a livetype cache. Work Windows machine with Adobe fonts remains a later `.exe` exercise for `%APPDATA%\Adobe\CoreSync\plugins\livetype\`. Do not block the Mac preview on either box.

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

Path is `livetype/.c/entitlements.xml` — **not** `livetype/entitlements.xml`.

The Python script looked in the wrong place and read attributes, so the 2025-09-19 run had **empty `xml_id` on all 1,391 rows** and guessed with fontTools. That run is a **volume check** (201 families, 1,391 fonts, ~216 MB of fonts), not a naming check.

Fallback: OpenType name table + `fvar`. Trust magic bytes (`OTTO` / `\x00\x01\x00\x00`) over Adobe’s `.otf` extension.

Keep Adobe’s `familyName` splits (CondorWide vs CondorCond, Adorn vs AdornS). Don’t merge them.

Copy with a read/write of the bytes (`copy_font`), not `fs::copy`. On APFS, `fs::copy` clonefiles and keeps Adobe’s mtimes, so a second GATHER looks like a renamed duplicate of the first. Dest files get *now* timestamps. Don’t sleep to fake the roll.

## UI (mockup `mockups/FONarch-UI-v2.png`)

Amber phosphor terminal. Pixel title **FONarch**. Gear (settings) top-left. Custom chrome: minimize + close top-right, plus Cmd+Q / Alt+F4. Left: GLSL wireframe landscape. Right: rolling list. Segmented LED bar. Status line. Two capsule buttons.

Settings fills the glass (titlebar stays). Two columns: THEME | SAVE LOCATION + ABOUT. About: crown (tinted from `--accent`) + **FONarch**, then `VERSION 0.9.0 by Mick Jay` (link https://mdeanjones.net/), `Licensed under Apache 2.0`, `NOT affiliated with Adobe.`, `FONT: Pet Me by Kreative Software`. Dock icon is `assets/identity/FONarch-icon-withBKG.svg` (phosphor `#FFB000` on black). Crown-on-the-window chrome is v2.

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
- Local clone: `~/Grok/AI CODING PROJECTS/FONarchive_app`
- Last user run: `~/Desktop/FONarchive/` from 2025-09-19 (also `FONarchive.zip` on Desktop)

## Out of scope for this app

Admin elevation, pip, visible `working/`, tray icon, user-swappable Winamp skin engine, scanning other users’ homes, decrypting `.e` blobs.
