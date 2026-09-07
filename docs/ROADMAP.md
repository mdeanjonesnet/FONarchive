# FONarch — tentative roadmap

Not a contract. Order can slip. 1.0 means “Mac and Windows both have a download,” not “every idea is in.”

## Now — 1.0 (shipped 2026-09-07)

Mac Apple Silicon `.dmg` (signed and notarized) **and** Windows `.exe` (unsigned NSIS) on GitHub Releases tag `v1.0.0`. GATHER / ARCHIVE, four CRT themes, custom save folder, About. GitHub is `mickjayofficial/FONarch`.

**Not in 1.0:** 1.1 bug reports, Polar accent slider, Authenticode, Intel Mac.

## Next — 1.1 (few days)

Locked 2026-09-06. Both platform downloads exist. Start when Michael opens a FONarch 1.1 session.

- **Bugs:** pixel insect next to the gear (generic GitHub issue, light prefill). `REPORT ERROR` on the status row only when a run actually failed (error + diagnostics prefilled). Browser form; user hits Submit. Markdown template, not a YAML issue form.
- **Accent:** Polar pay-what-you-want (min $1, suggested $3) mints a `FONARCH_…` key. SETTINGS: DONATE + paste key. Hue slider unlocks. Four CRT themes still pick `--bg`.
- Polar is Merchant of Record. Existing Stripe login is **payouts only** (Connect). No Stripe keys in the app. No Ko-fi. No honor-system checkbox.

## Won't

- Intel Mac
- Crown sitting on the window
- `fonarch.com` (future public home is JMDCO, not a FONarch task)
- Ko-fi

## Not this product

A “FontBase on steroids” AI font manager is a **separate** app and repo.

## Python

Frozen. Tag `python-v1`, branch `archive/python-v1`, MIT. Do not relicense it. Do not put `fonarchive_manager.py` on the new `main`.
