# YT-074 — Real Chewing IME Evidence

Captured on 2026-10-13 from an owned, headful Firefox 157.0.1 window on Ubuntu 26.04.1 LTS / Xfce X11. The images are original GDK window-region captures, visually inspected without image editing. Fcitx5 5.1.19-1 and Chewing 5.1.10-1 received native OS key events; WebDriver Unicode typing, paste and synthetic composition were not used to produce the Chinese text.

See the [desktop record](../../YT-074-DESKTOP-ACCEPTANCE.md) and [sanitized observations](../../YT-074-DESKTOP-EVIDENCE.json). The Board uses only synthetic identities and an isolated database. No login code, Cookie or CSRF value appears in these images. The small desktop tooltip is incidental desktop chrome, not Board content.

| Image | Actual state |
|---|---|
| [01: candidates](01-real-chewing-candidates.png) | Chewing candidate window after `s u 3 Down`; candidate 2 is selected next. |
| [02: committed form](02-real-ime-committed-form.png) | Committed title and multiline description, with visible textarea focus. |
| [03: persisted detail](03-ime-persisted-detail.png) | Version 1, exact title/description and transactional history after native submission. |
| [04: validation](04-validation-retained.png) | HTTP 422 for a closed-to-active transition, preserving the Chinese draft. |
| [06: conflict](06-conflict-page.png) | Settled HTTP 409 page, retained Chinese draft, conflict notice and current-record link; native Tab exposes visible skip-link focus. |
| [07: field error](07-field-validation.png) | Whitespace-title HTTP 422, linked title error and retained Chinese description; visible keyboard focus after skip-to-main. |
| [08: authoritative version](08-authoritative-version.png) | Version 3 remains authoritative after the stale version-2 submission. |

Capture 05 was taken before the native window finished painting the navigation. It remains at `target/m11-board/desktop-20261013/05-before-paint-not-acceptance.png` as an excluded diagnostic, not a second acceptance screenshot (SHA-256 `bd7923f19458caa3f16efe01cf29432b5d8d5e14534922b0b78ab3d87caa2125`).

## SHA-256

```text
de6f160a84fa96fa30d0321c4f6357d84ed1b702b67f3c35bc0bfa2548e76b80  01-real-chewing-candidates.png
be88e225fc5e1dff5569e80d5af4b33a9f3c6fab67f5f4a3cdfc3a1449fe5260  02-real-ime-committed-form.png
c398903b3f9721680ec12e7afb9fc7236de50c806a2bbe42076ecb720e428a72  03-ime-persisted-detail.png
113c5612d44d601931c9fa38b4f1ca1db36838d127cd71f12264706cb2772c1e  04-validation-retained.png
40de65a3228bead2609adac39f0bfe366a0055f2aff173d878ac36c0631483c9  06-conflict-page.png
bd2c9e82dc7ef4c1191bcec2660c01af256b638754331554074d60ef5b225397  07-field-validation.png
8264d6523a7787c3e1dbea30abba285a22a473f2977c20d8e8f0b6258229ccd1  08-authoritative-version.png
```
