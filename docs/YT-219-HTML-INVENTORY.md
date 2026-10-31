# YT-219 — HTML element and attribute inventory

Date: 2026-10-31. Status: independently accepted source inventory for the [compatibility contract](YT-219-HTML-DOM-CONTRACT.md); no implementation claim.

## Reference snapshot and interpretation

Source: [WHATWG HTML conforming-element/attribute indices](https://html.spec.whatwg.org/multipage/indices.html),
header date **30 October2026**, retrieved2026-10-31. Raw source SHA256
`0a865ea90576d2976db607d324b1e27de14a7d5f53f96da3f63c209ea288181d`.
The raw488,697-byte page and extracted rows are retained under
`target/yt219-contract-20261031/`; this is a dated inventory, not a floating
promise to implement future standards additions.

**Coverage:**111 element-index rows expand to113 named HTML elements, two
foreign roots and one custom-element family (116 entries below).180 ordinary
attribute/applicability rows cover144 distinct names;89 event-handler rows
are separately listed and rejected. Attribute rows preserve distinct standard
scopes rather than collapsing repeated names. Sources describe HTML semantics;
dispositions below are Yetlet decisions, not standards conformance claims.

Current source is the committed YT-219 planning baseline. `SSR G` means the
serializer accepts a syntactically valid name/tag with its existing escaping,
duplicate/forbidden/URL/void/control checks; it does **not** validate complete
HTML applicability or resource policies. DOM current status is conditional on
the containing element itself being one of the38 admitted tags.

| Code | Meaning |
|---|---|
| C | Currently accepted with existing limits; not newly implemented here |
|220 | Selected YT-220 ordinary-content capability; not implemented yet |
|221 | Selected YT-221 explicit navigation/image capability; not implemented yet |
| W | Document/head/asset writer owns it, outside mounted subtrees |
| D | Deferred with the stated reason; not permanently forbidden |
| X | Excluded from the ordinary DOM View path |

## Element matrix

| Element | SSR now | DOM now | Selected disposition | Reason |
|---|---|---|---|---|
| `a` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `abbr` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `address` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `area` | G | No | D | Image-map geometry, activation and URL policy need a consumer |
| `article` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `aside` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `audio` | G | No | D | Media loading, source selection and accessibility lifecycle need a consumer |
| `b` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `base` | G | No | W | Initial document/head writer owns this; outside subtree mount |
| `bdi` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `bdo` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `blockquote` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `body` | G | No | W | Initial document/head writer owns this; outside subtree mount |
| `br` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `button` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `canvas` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `caption` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `cite` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `code` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `col` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `colgroup` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `data` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `datalist` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `dd` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `del` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `details` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `dfn` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `dialog` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `div` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `dl` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `dt` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `em` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `embed` | Rejected | No | X | Executable/embedded content remains rejected in ordinary Views |
| `fieldset` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `figcaption` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `figure` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `footer` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `form` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `h1` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `h2` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `h3` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `h4` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `h5` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `h6` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `head` | G | No | W | Initial document/head writer owns this; outside subtree mount |
| `header` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `hgroup` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `hr` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `html` | G | No | W | Initial document/head writer owns this; outside subtree mount |
| `i` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `iframe` | Rejected | No | X | Executable/embedded content remains rejected in ordinary Views |
| `img` | G | No | 221 | Explicit image policy; disabled through legacy/default entry points |
| `input` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `ins` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `kbd` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `label` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `legend` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `li` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `link` | G | No | W | Initial document/head writer owns this; outside subtree mount |
| `main` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `map` | G | No | D | Image-map geometry, activation and URL policy need a consumer |
| `mark` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `MathML math` | No namespace/family contract | No | D | Foreign namespace/custom-element behavior is outside the shared HTML model |
| `menu` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `meta` | G | No | W | Initial document/head writer owns this; outside subtree mount |
| `meter` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `nav` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `noscript` | G | No | D | Parser/scripting-dependent fallback differs from direct DOM creation |
| `object` | Rejected | No | X | Executable/embedded content remains rejected in ordinary Views |
| `ol` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `optgroup` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `option` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `output` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `p` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `picture` | G | No | D | Responsive/media source grammar deferred; first image slice is img only |
| `pre` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `progress` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `q` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `rp` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `rt` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `ruby` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `s` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `samp` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `script` | Rejected | No | X | Executable/embedded content remains rejected in ordinary Views |
| `search` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `section` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `select` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `selectedcontent` | G | No | D | Separate template/shadow/select composition semantics are not implemented |
| `slot` | G | No | D | Separate template/shadow/select composition semantics are not implemented |
| `small` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `source` | G | No | D | Responsive/media source grammar deferred; first image slice is img only |
| `span` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `strong` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `style` | Rejected | No | X | Executable/embedded content remains rejected in ordinary Views |
| `sub` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `summary` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `sup` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `SVG svg` | No namespace/family contract | No | D | Foreign namespace/custom-element behavior is outside the shared HTML model |
| `table` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `tbody` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `td` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `template` | G | No | D | Separate template/shadow/select composition semantics are not implemented |
| `textarea` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `tfoot` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `th` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `thead` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `time` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `title` | G | No | W | Initial document/head writer owns this; outside subtree mount |
| `tr` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `track` | G | No | D | Media loading, source selection and accessibility lifecycle need a consumer |
| `u` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `ul` | G | C | C | Existing DOM admission; existing scoped attributes and safety limits still apply |
| `var` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `video` | G | No | D | Media loading, source selection and accessibility lifecycle need a consumer |
| `wbr` | G | No | 220 | Selected ordinary HTML; scoped attributes/structures in the contract |
| `autonomous custom elements` | No namespace/family contract | No | D | Foreign namespace/custom-element behavior is outside the shared HTML model |

Disposition totals: 220=51, 221=1, C=38, D=14, W=7, X=5.

The HTML serializer can lexically emit `math`/`svg` and hyphenated custom names;
that is not namespace construction, parser/direct-DOM equivalence or permission
to execute custom-element definitions. Obsolete names absent from this conforming
index are not selected (for example `font`, `center`, `marquee`, `frame`,
`frameset`, `param`); existing broad SSR admission is not silently tightened.

## Ordinary attribute/applicability matrix

`G` in DOM now means the current global-name allowlist, even where HTML normally
scopes that name more narrowly. `C: tags` reports the source scoped-name check;
intersect it with the standard scope column to locate a conforming use. Existing
global admission of `type`, `for`, `placeholder`, `maxlength`, `autocomplete` is
preserved for compatibility. New uses get the contract's scoped rules.

All ordinary names below are currently SSR G except `style`/`srcdoc`, which are
explicitly forbidden. The five-name SSR URL check is documented in the contract;
other URL-shaped attributes do not thereby gain a safe URL guarantee.

| Attribute | Standard applicability snapshot | DOM now | Selected disposition | Reason |
|---|---|---|---|---|
| `abbr` | th | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `accept` | input | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `accept-charset` | form | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `accesskey` | HTML elements | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `action` | form | C: form | C | Existing form-only same-origin policy retained; no broader form navigation |
| `allow` | iframe | No | X / D | Applies to excluded executable/embedded hosts or deferred resource hosts |
| `allowfullscreen` | iframe | No | X / D | Applies to excluded executable/embedded hosts or deferred resource hosts |
| `alpha` | input | No | D | New browser-dependent behavior needs selected target/consumer evidence |
| `alt` | area; img; input | No | 221 | img requires alt; area/input-image contexts deferred |
| `as` | link | No | W | Document/head/asset writer scope, not owned-subtree DOM |
| `async` | script | No | X / D | Applies to excluded executable/embedded hosts or deferred resource hosts |
| `autocapitalize` | HTML elements | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `autocomplete` | form | G | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `autocomplete` | input; select; textarea | G | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `autocorrect` | HTML elements | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `autofocus` | HTML elements | No | D | Focus/command/popover/modal behavior needs explicit ownership and a consumer |
| `autoplay` | audio; video | No | D | Specialized/resource/composition context requires a separate selected consumer and contract |
| `blocking` | link; script; style | No | W | Document/head/asset writer scope, not owned-subtree DOM |
| `charset` | meta | No | W | Document/head/asset writer scope, not owned-subtree DOM |
| `checked` | input | C: input | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `cite` | blockquote; del; ins; q | No | D | Additional request/navigation/submission/metadata policy not selected; preserve currently accepted names only |
| `class` | HTML elements | G | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `closedby` | dialog | No | D | Focus/command/popover/modal behavior needs explicit ownership and a consumer |
| `color` | link | No | W | Document/head/asset writer scope, not owned-subtree DOM |
| `colorspace` | input | No | D | New browser-dependent behavior needs selected target/consumer evidence |
| `cols` | textarea | C: textarea | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `colspan` | td; th | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `command` | button | No | D | Focus/command/popover/modal behavior needs explicit ownership and a consumer |
| `commandfor` | button | No | D | Focus/command/popover/modal behavior needs explicit ownership and a consumer |
| `content` | meta | No | W | Document/head/asset writer scope, not owned-subtree DOM |
| `contenteditable` | HTML elements | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `controls` | audio; video img; | No | D | Specialized/resource/composition context requires a separate selected consumer and contract |
| `coords` | area | No | D | Specialized/resource/composition context requires a separate selected consumer and contract |
| `crossorigin` | audio; img; link; script; video | No | D | Additional request/navigation/submission/metadata policy not selected; preserve currently accepted names only |
| `data` | object | No | X / D | Applies to excluded executable/embedded hosts or deferred resource hosts |
| `datetime` | del; ins | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `datetime` | time | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `decoding` | img | No | 221 | Selected img/a scope only; other listed element contexts remain deferred/excluded |
| `default` | track | No | D | Specialized/resource/composition context requires a separate selected consumer and contract |
| `defer` | script | No | X / D | Applies to excluded executable/embedded hosts or deferred resource hosts |
| `dir` | HTML elements | G | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `dir` | bdo | G | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `dirname` | input; textarea | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `disabled` | button; input; optgroup; option; select; textarea; form-associated custom elements | C: button, fieldset, input, option, select, textarea | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `disabled` | fieldset | C: button, fieldset, input, option, select, textarea | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `disabled` | link | C: button, fieldset, input, option, select, textarea | W / D | Attribute name alone does not admit document, specialized or excluded element scopes |
| `download` | a; area | No | D | Additional request/navigation/submission/metadata policy not selected; preserve currently accepted names only |
| `draggable` | HTML elements | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `enctype` | form | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `enterkeyhint` | HTML elements | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `fetchpriority` | img; link; script | No | 221 | Selected img/a scope only; other listed element contexts remain deferred/excluded |
| `for` | label | G | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `for` | output | G | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `for` | template | G | W / D | Attribute name alone does not admit document, specialized or excluded element scopes |
| `form` | button; fieldset; input; object; output; select; textarea; form-associated custom elements | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `formaction` | button; input | No | D | Additional request/navigation/submission/metadata policy not selected; preserve currently accepted names only |
| `formenctype` | button; input | No | D | Additional request/navigation/submission/metadata policy not selected; preserve currently accepted names only |
| `formmethod` | button; input | No | D | Additional request/navigation/submission/metadata policy not selected; preserve currently accepted names only |
| `formnovalidate` | button; input | No | D | Additional request/navigation/submission/metadata policy not selected; preserve currently accepted names only |
| `formtarget` | button; input | No | D | Additional request/navigation/submission/metadata policy not selected; preserve currently accepted names only |
| `headers` | td; th | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `headingoffset` | HTML elements | No | D | New browser-dependent behavior needs selected target/consumer evidence |
| `headingreset` | HTML elements | No | D | New browser-dependent behavior needs selected target/consumer evidence |
| `height` | canvas; embed; iframe; img; input; object; source (in picture); video | No | 220 / 221 | 220 canvas dimensions;221 img dimensions; other listed hosts deferred/excluded |
| `hidden` | HTML elements | G | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `high` | meter | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `href` | a; area | C: a | 221 | a only: existing default retained; external origins require explicit navigation policy; area deferred |
| `href` | link | C: a | W / D | Document writer or unselected form/map scope; no widening from link policy |
| `href` | base | C: a | W / D | Document writer or unselected form/map scope; no widening from link policy |
| `hreflang` | a; link | No | 221 | Selected img/a scope only; other listed element contexts remain deferred/excluded |
| `http-equiv` | meta | No | W | Document/head/asset writer scope, not owned-subtree DOM |
| `id` | HTML elements | G | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `imagesizes` | link | No | W | Document/head/asset writer scope, not owned-subtree DOM |
| `imagesrcset` | link | No | W | Document/head/asset writer scope, not owned-subtree DOM |
| `inert` | HTML elements | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `inputmode` | HTML elements | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `integrity` | link; script | No | W | Document/head/asset writer scope, not owned-subtree DOM |
| `is` | HTML elements | No | X | Customized built-in execution is not ordinary element creation |
| `ismap` | img | No | D | Additional request/navigation/submission/metadata policy not selected; preserve currently accepted names only |
| `itemid` | HTML elements | No | D | Microdata vocabulary/URL semantics are not selected; SSR metadata remains separate |
| `itemprop` | HTML elements | No | D | Microdata vocabulary/URL semantics are not selected; SSR metadata remains separate |
| `itemref` | HTML elements | No | D | Microdata vocabulary/URL semantics are not selected; SSR metadata remains separate |
| `itemscope` | HTML elements | No | D | Microdata vocabulary/URL semantics are not selected; SSR metadata remains separate |
| `itemtype` | HTML elements | No | D | Microdata vocabulary/URL semantics are not selected; SSR metadata remains separate |
| `kind` | track | No | D | Specialized/resource/composition context requires a separate selected consumer and contract |
| `label` | optgroup; option; track | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `lang` | HTML elements | G | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `list` | input | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `loading` | iframe; img; audio; video | No | 221 | Selected img/a scope only; other listed element contexts remain deferred/excluded |
| `loop` | audio; video | No | D | Specialized/resource/composition context requires a separate selected consumer and contract |
| `low` | meter | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `max` | input | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `max` | meter; progress | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `maxlength` | input; textarea | G | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `media` | link; meta; source; style | No | D | Specialized/resource/composition context requires a separate selected consumer and contract |
| `method` | form | C: form | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `min` | input | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `min` | meter | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `minlength` | input; textarea | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `multiple` | input; select | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `muted` | audio; video | No | D | Specialized/resource/composition context requires a separate selected consumer and contract |
| `name` | button; fieldset; input; output; select; textarea; form-associated custom elements | C: button, form, input, select, textarea | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `name` | details | C: button, form, input, select, textarea | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `name` | form | C: button, form, input, select, textarea | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `name` | iframe; object | C: button, form, input, select, textarea | W / D | Attribute name alone does not admit document, specialized or excluded element scopes |
| `name` | map | C: button, form, input, select, textarea | W / D | Attribute name alone does not admit document, specialized or excluded element scopes |
| `name` | meta | C: button, form, input, select, textarea | W / D | Attribute name alone does not admit document, specialized or excluded element scopes |
| `name` | slot | C: button, form, input, select, textarea | W / D | Attribute name alone does not admit document, specialized or excluded element scopes |
| `nomodule` | script | No | X / D | Applies to excluded executable/embedded hosts or deferred resource hosts |
| `nonce` | HTML elements | No | W | CSP/asset writer controls nonces; not a mount option |
| `novalidate` | form | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `open` | details | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `open` | dialog | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `optimum` | meter | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `pattern` | input | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `ping` | a; area | No | D | Additional request/navigation/submission/metadata policy not selected; preserve currently accepted names only |
| `placeholder` | input; textarea | G | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `playsinline` | video | No | D | Specialized/resource/composition context requires a separate selected consumer and contract |
| `popover` | HTML elements | No | D | Focus/command/popover/modal behavior needs explicit ownership and a consumer |
| `popovertarget` | button; input | No | D | Focus/command/popover/modal behavior needs explicit ownership and a consumer |
| `popovertargetaction` | button; input | No | D | Focus/command/popover/modal behavior needs explicit ownership and a consumer |
| `poster` | video | No | D | Specialized/resource/composition context requires a separate selected consumer and contract |
| `preload` | audio; video | No | D | Specialized/resource/composition context requires a separate selected consumer and contract |
| `readonly` | input; textarea | C: input, textarea | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `readonly` | form-associated custom elements | C: input, textarea | W / D | Attribute name alone does not admit document, specialized or excluded element scopes |
| `referrerpolicy` | a; area; iframe; img; link; script | No | 221 | Selected img/a scope only; other listed element contexts remain deferred/excluded |
| `rel` | a; area | No | 221 | Selected img/a scope only; other listed element contexts remain deferred/excluded |
| `rel` | link | No | W / D | Document writer or unselected form/map scope; no widening from link policy |
| `required` | input; select; textarea | C: input, select, textarea | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `reversed` | ol | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `rows` | textarea | C: textarea | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `rowspan` | td; th | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `sandbox` | iframe | No | X / D | Applies to excluded executable/embedded hosts or deferred resource hosts |
| `scope` | th | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `selected` | option | C: option | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `shadowrootclonable` | template | No | D | Specialized/resource/composition context requires a separate selected consumer and contract |
| `shadowrootcustomelementregistry` | template | No | D | Specialized/resource/composition context requires a separate selected consumer and contract |
| `shadowrootdelegatesfocus` | template | No | D | Specialized/resource/composition context requires a separate selected consumer and contract |
| `shadowrootmode` | template | No | D | Specialized/resource/composition context requires a separate selected consumer and contract |
| `shadowrootserializable` | template | No | D | Specialized/resource/composition context requires a separate selected consumer and contract |
| `shadowrootslotassignment` | template | No | D | Specialized/resource/composition context requires a separate selected consumer and contract |
| `shape` | area | No | D | Specialized/resource/composition context requires a separate selected consumer and contract |
| `size` | input; select | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `sizes` | link | No | W | Document/head/asset writer scope, not owned-subtree DOM |
| `sizes` | img; source | No | D | Responsive image candidate/sizes grammar deferred; not a raw-string bypass |
| `slot` | HTML elements | No | D | Specialized/resource/composition context requires a separate selected consumer and contract |
| `span` | col; colgroup | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `spellcheck` | HTML elements | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `src` | audio; embed; iframe; img; input; script; source (in video or audio); track; video | No | 221 | img only; media, executable and embedded sources remain unselected |
| `srcdoc` | iframe | No | X | Embedded markup remains excluded |
| `srclang` | track | No | D | Specialized/resource/composition context requires a separate selected consumer and contract |
| `srcset` | img; source | No | D | Responsive image candidate/sizes grammar deferred; not a raw-string bypass |
| `start` | ol | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `step` | input | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `style` | HTML elements | No | X | Inline style remains excluded; use external CSS |
| `tabindex` | HTML elements | G | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `target` | a; area | No | 221 | Selected img/a scope only; other listed element contexts remain deferred/excluded |
| `target` | base | No | W / D | Document writer or unselected form/map scope; no widening from link policy |
| `target` | form | No | W / D | Document writer or unselected form/map scope; no widening from link policy |
| `title` | HTML elements | G | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `title` | abbr; dfn | G | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `title` | input | G | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `title` | link | G | W / D | Attribute name alone does not admit document, specialized or excluded element scopes |
| `title` | link; style | G | W / D | Attribute name alone does not admit document, specialized or excluded element scopes |
| `translate` | HTML elements | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `type` | a; link | G | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `type` | button | G | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `type` | embed; object; source | G | W / D | Attribute name alone does not admit document, specialized or excluded element scopes |
| `type` | input | G | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `type` | ol | G | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `type` | script | G | W / D | Attribute name alone does not admit document, specialized or excluded element scopes |
| `usemap` | img | No | D | Additional request/navigation/submission/metadata policy not selected; preserve currently accepted names only |
| `value` | button; option | C: button, input, option | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `value` | data | C: button, input, option | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `value` | input | C: button, input, option | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `value` | li | C: button, input, option | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `value` | meter; progress | C: button, input, option | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `width` | canvas; embed; iframe; img; input; object; source (in picture); video | No | 220 / 221 | 220 canvas dimensions;221 img dimensions; other listed hosts deferred/excluded |
| `wrap` | textarea | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |
| `writingsuggestions` | HTML elements | No | 220 | Preserve existing admission; add scoped ordinary-content/native-control uses only |

## Event-handler content attributes

All89 indexed names below are **X** in the DOM path and rejected by SSR's
`on*` rule. Use explicit owned Rust/web-sys listeners instead. This is a
complete name inventory for this snapshot, not permission for an `on*` escape.

| Names |
|---|
| `onafterprint`, `onauxclick`, `onbeforeinput`, `onbeforematch`, `onbeforeprint`, `onbeforeunload` |
| `onbeforetoggle`, `onblur`, `oncancel`, `oncanplay`, `oncanplaythrough`, `onchange` |
| `onclick`, `onclose`, `oncommand`, `oncontextlost`, `oncontextmenu`, `oncontextrestored` |
| `oncopy`, `oncuechange`, `oncut`, `ondblclick`, `ondrag`, `ondragend` |
| `ondragenter`, `ondragleave`, `ondragover`, `ondragstart`, `ondrop`, `ondurationchange` |
| `onemptied`, `onended`, `onerror`, `onfocus`, `onformdata`, `onhashchange` |
| `oninput`, `oninvalid`, `onkeydown`, `onkeypress`, `onkeyup`, `onlanguagechange` |
| `onload`, `onloadeddata`, `onloadedmetadata`, `onloadstart`, `onmessage`, `onmessageerror` |
| `onmousedown`, `onmouseenter`, `onmouseleave`, `onmousemove`, `onmouseout`, `onmouseover` |
| `onmouseup`, `onoffline`, `ononline`, `onpagehide`, `onpagereveal`, `onpageshow` |
| `onpageswap`, `onpaste`, `onpause`, `onplay`, `onplaying`, `onpopstate` |
| `onprogress`, `onratechange`, `onreset`, `onresize`, `onrejectionhandled`, `onscroll` |
| `onscrollend`, `onsecuritypolicyviolation`, `onseeked`, `onseeking`, `onselect`, `onslotchange` |
| `onstalled`, `onstorage`, `onsubmit`, `onsuspend`, `ontimeupdate`, `ontoggle` |
| `onunhandledrejection`, `onunload`, `onvolumechange`, `onwaiting`, `onwheel` |

## Supplementary current names and attribute families

The index does not enumerate the following current DOM family members:

| Family | Current names | Selected boundary |
|---|---|---|
| ARIA | `aria-busy`, `aria-controls`, `aria-current`, `aria-describedby`, `aria-expanded`, `aria-hidden`, `aria-invalid`, `aria-label`, `aria-labelledby`, `aria-live`, `aria-pressed` | Retain finite names; do not claim full ARIA semantics or admit arbitrary prefixes |
| Data | `data-completed`, `data-id`, `data-meta`, `data-select`, `data-selected`, `data-title` | Retain finite names; reserve data-yetlet-* |
| Role | `role` | Retain; application owns valid role/state/reference combinations |

Other `aria-*`/`data-*` names remain deferred until separately selected. The
existing `InertAttributeName` slot vocabulary is broader than DOM admission;
its validation alone does not authorize DOM attributes or imply identical policy.
Colon-qualified names such as `xml:lang`/`xlink:href` do not acquire namespace
support; the HTML renderer's identifier check rejects colons despite its old
URL-name list mentioning xlink:href. Unknown attributes/tags remain rejected by
DOM; no generic attribute spread, inline CSS or raw markup is introduced.
