# Lab Operations Board — Actual Browser Captures

Captured during the repaired Chrome 155.0.8059.39 journey on 2026-10-13. These are real rendered pages, not generated mockups. All records and identities are synthetic. No login code, Cookie or CSRF value is visible. The isolated screenshot database and browser were removed after verification; normal startup begins with its own empty/persisted database.

| Capture | Surface | Viewport |
|---|---|---|
| [01-work-items.png](01-work-items.png) | Persisted records, real counts, filters, sorting and project navigation | 1440 x 1000 CSS pixels |
| [02-item-history.png](02-item-history.png) | Record detail and transactionally stored history | 1440 x 1000 |
| [03-conflict-draft.png](03-conflict-draft.png) | Actual HTTP 409, preserved draft and latest-version link | 1280 x 960 |
| [04-narrow-live-status.png](04-narrow-live-status.png) | Narrow navigation, stacked form controls and paused live state | 390 x 844 |
| [05-edit-work-item.png](05-edit-work-item.png) | Native edit form, selected values and submit/reset actions | 1440 x 1000 |

The current session visually inspected all five images. Text/labels are readable, the selected controls and stored values are visible, desktop navigation remains separate from the task surface, the narrow layout stacks controls, and the conflict notice does not erase the user's draft. Long pages scroll normally; these are viewport captures, not claims that every page fits in one screen. The simple Bootstrap design and explicit Unix-second history timestamps are retained; this is not a new visual redesign, WCAG conformance or spoken screen-reader signoff.

See [startup instructions](../../BOARD-START.md) and [qualification scope/results](../../YT-074-QUALIFICATION.md).
