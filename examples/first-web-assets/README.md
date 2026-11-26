# First CSS and JavaScript

This source overlay adds a message form to the plain Native scaffold using
Yetlet SDK `0.9.0-alpha.9` and CLI `0.4.2`. Follow
[First App](../../docs/FIRST-APP.md#add-your-first-css-and-javascript) to copy
the three files into the generated app and run it with its existing Cargo lock.

| File | Role |
| --- | --- |
| [`src/main.rs`](src/main.rs) | Links both assets in the document head, serves them at two fixed routes, and handles the form POST. |
| [`src/app.css`](src/app.css) | Styles the form and its server-rendered result. |
| [`src/app.js`](src/app.js) | Shows a local text preview as you type. |

The browser submits an ordinary form and the server returns a complete page
with the submitted text. The fixed document template inserts only the
result fragment rendered by Yetlet, which escapes the submitted value via
`View::text`. The preview inserts the same text with `textContent`.

The app embeds the two asset files at build time. Each route supplies its media
type and serves the asset from the page's origin under `DEFAULT_CSP`. Edit either
file and rebuild to see a change. This example follows the
[application-owned wrapper decision](../../docs/ASSETS.md#m11-native-css-proof-and-retained-wrapper-decision).
