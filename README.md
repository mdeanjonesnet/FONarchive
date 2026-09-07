```text
################################################################################################################
#                                                                                                              #
#                                                      ###                                                     #
#                                                   ########                                                   #
#                              ###               ##############               ###                              #
#                              ########        ###################       ########                              #
#                               ############ ####################################                              #
#                               ############ ###################################                               #
#                                 ########## ######## #####    ###############                                 #
#                                 ######## #######################                                             #
#                                   ##### ##################################                                   #
#                                   ########################################                                   #
#                                   ###############           ##############                                   #
#                                   ##############                    ######                                   #
#                                   #########           ###################                                    #
#                                   ##############################                                             #
#                                                                                                              #
# ############      ########      ####    ####                                                                 #
# ####            ####    ####    ######  ####                                                    ####         #
# ####            ####    ####    ############      ########      ##########        ########      ####         #
# ########        ####    ####    ############            ####    ####    ####    ####            ##########   #
# ####            ####    ####    ####  ######      ##########    ####            ####            ####    #### #
# ####            ####    ####    ####    ####    ####    ####    ####            ####            ####    #### #
# ####              ########      ####    ####      ##########    ####              ########      ####    #### #
#                                                                                                              #
#                                  hidden adobe fonts. named. sorted. yours.                                   #
#                              v1.0.0  mac apple silicon + windows  like monarch                               #
#                                                                                                              #
################################################################################################################
```

Tiny app. Finds Creative Cloud’s livetype cache, names the real OpenType, and either drops a folder on your Desktop or zips it. Source cache stays read-only.

**1.0 is out.** Mac **and** Windows both have a download.

- **Mac (Apple Silicon):** [FONarch_1.0.0_aarch64.dmg](https://github.com/mickjayofficial/FONarch/releases/download/v1.0.0/FONarch_1.0.0_aarch64.dmg) — signed and notarized. Open the disk image, drag FONarch to Applications.
- **Windows:** [FONarch_1.0.0_x64-setup.exe](https://github.com/mickjayofficial/FONarch/releases/download/v1.0.0/FONarch_1.0.0_x64-setup.exe) — unsigned. SmartScreen may say Windows protected your PC → **More info** → **Run anyway**.

Intel Macs are not in this drop.

```text
  --[ WHY ]-------------------------------------
```

Adobe Fonts land in a hidden livetype cache. You pay for them. Your own font manager can’t see them. Reinstalling the whole set through Creative Cloud after a dead drive or a new machine is miserable.

FONarch was built so you can **organize those fonts in a manager you actually like**, and keep a **named, sorted archive** — folder or zip — that you dump onto the next box and use immediately, instead of clicking Activate a thousand times.

```text
  --[ NOTICE ]----------------------------------
```

**This is only for archiving and personal use of fonts you already pay for.**  
It is not a pirate tool. Do not redistribute Adobe’s fonts.  
**FONarch is NOT associated with, endorsed by, or affiliated with Adobe Inc.**

```text
  --[ INSTALL ]---------------------------------
```

**Mac**

1. Download the `.dmg` (link at the top, or [Releases](https://github.com/mickjayofficial/FONarch/releases/latest)).
2. Drag **FONarch** onto **Applications**.
3. Open. macOS may once say it came from the internet — that’s normal.

**Windows**

1. Download the `.exe` (link at the top, or [Releases](https://github.com/mickjayofficial/FONarch/releases/latest)).
2. If SmartScreen appears: **More info** → **Run anyway**.
3. Install for this user. GATHER / ARCHIVE from the window.

```text
  --[ KEYS ]------------------------------------
```

|          | |
|----------|-|
| **GATHER**  | dated folder, fonts renamed and sorted by family |
| **ARCHIVE** | same, then a `.zip` — no leftover folder |
| **gear**    | themes, save location, About, Quit |

Each button is a full run. ARCHIVE does not need GATHER first. Desktop is the default; the gear can point it elsewhere. Cmd+Q / Alt+F4 also quits.

```text
  --[ WON'T ]-----------------------------------
```

Won’t write livetype. Won’t decrypt `.e` blobs. Won’t scan other users. Won’t ask for Python, pip, or an admin password.

```text
  --[ MAP ]-------------------------------------
```

Tentative: [docs/ROADMAP.md](docs/ROADMAP.md). **1.0** is Mac Apple Silicon + Windows. Next is 1.1: GitHub bug reports (titlebar insect + status-row on failure) and Polar accent unlock. No Intel Mac, no crown-on-window, no Ko-fi, no `fonarch.com`. Python freeze is tag [`python-v1`](https://github.com/mickjayofficial/FONarch/releases/tag/python-v1) (MIT). This rewrite is Apache 2.0.

```text
  --[ HACK ]------------------------------------
```

`ui/` is the window. `assets/` is fonts and the crown. Node 22 + rustc.

`npm install && npm run tauri -- dev`  
Headless: `cargo run -p fonarch-core --bin gather`

```text
  --[ LICENSE ]---------------------------------
```

[Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0). See NOTICE above for the Adobe line.  
`python-v1` stays MIT.  
Pet Me / Pet Me 64 © [Kreative Software](https://www.kreativekorp.com/) (`assets/fonts/FreeLicense.txt`).
