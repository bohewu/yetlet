# yetlet::html

**Experimental `0.9.0-alpha.5`, unpublished.** Requires Rust 1.99 or newer. This is one of Yetlet's sixteen library candidates; use the local checkout rather than assuming a registry release. APIs are pre-public, and bounded prototype verification is not production qualification.

`yetlet::html` serializes a `yetlet-core::View` into escaped HTML on native Rust. Its only dependency is `yetlet::view`; it installs no HTTP server or browser runtime.

## Render a View

Call `render_to_string(&view)` to receive `Result<String, RenderError>`. Text and attribute values are escaped; `escape_text` is also available for text escaping. Build the View using `Element` and `View` from core. The complete [Hello World handler](../../../examples/hello-world/src/lib.rs) uses this path through the server adapter. From the repository root:

```bash
cargo run --locked -p yetlet-hello-world
```

Open `http://127.0.0.1:3003/hello/World`. Use `yetlet-server-axum::Page` when you need a complete HTTP document with strict CSP; this serializer itself returns a fragment and owns neither response headers nor document metadata.

## Policy and errors

[YT-110](../../../docs/YT-110-DOM-CAPABILITIES.md) adds duplicate-name and selected control-child errors to the pre-public `RenderError` contract. Supply each attribute once, do not nest forms inside a View, and use text children for textarea/option. The serializer normalizes textarea CRLF/CR to LF and preserves an initial newline after HTML parsing; callers must remove any former manually prepended newline sentinel. Live values, selectedness and reset are browser/application behavior, not a serializer binding runtime.

`RenderError` distinguishes invalid tag/attribute names, forbidden attributes, unsafe URL schemes and children on void elements. Names follow the lowercase identifier policy; `script`, `style`, `iframe`, `object` and `embed` tags are rejected. Attributes beginning with `on`, plus `style` and `srcdoc`, are forbidden. Selected URL attributes reject script/data schemes conservatively, including whitespace/control obfuscation. There is no raw HTML escape hatch.

The serializer borrows a View for the call and returns an owned String; no mount or disposal lifetime exists. It buffers the complete fragment rather than streaming. Escaping is not a complete URL-policy engine or an accessibility checker: callers still choose semantic markup and application URL policy. The HTML and DOM subsets differ, so successful HTML serialization does not prove browser mount compatibility.

[Incremental SSR research](../../../docs/YT-021-STREAMING-SSR-RESEARCH.md) is a separate app-owned prototype; it does not add streaming or suspense to this crate.

## License and compatibility

Dual-licensed under [MIT](../../../LICENSE-MIT) or [Apache-2.0](../../../LICENSE-APACHE). The privately prepared `0.1.x` policy preserves documented behavior and Rust 1.99 in patches; public activation remains blocked. See [compatibility](../../../docs/COMPATIBILITY.md), [API classifications](../../../docs/API-CONTRACTS.md) and [developer guides](../../../docs/README.md). Links to companion packages and records assume this checkout; they do not imply public hosting or distribution.
