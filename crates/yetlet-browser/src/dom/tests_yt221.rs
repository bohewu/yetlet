use super::{
    DomOptions, DomPolicyError, ImagePolicy, NavigationPolicy, validate, validate_with_options,
};
use yetlet::view::{Element, View};

fn navigation() -> DomOptions {
    DomOptions::default().with_navigation_policy(NavigationPolicy::HttpsOrigins(vec![
        "https://docs.example.invalid".into(),
    ]))
}
fn image(src: &str) -> View {
    Element::new("img")
        .attribute("src", src)
        .attribute("alt", "")
        .into()
}

#[test]
fn yt221_defaults_and_independent_policy_values() {
    let options = DomOptions::default();
    assert_eq!(options.navigation_policy(), &NavigationPolicy::SameOrigin);
    assert_eq!(options.image_policy(), &ImagePolicy::Disabled);
    let changed = options.clone().with_image_policy(ImagePolicy::SameOrigin);
    assert_eq!(changed.navigation_policy(), options.navigation_policy());
    assert_eq!(
        changed.clone().with_image_policy(ImagePolicy::Disabled),
        options
    );
    let view = Element::new("a").attribute("href", "/docs#start").into();
    assert_eq!(validate(&view), validate_with_options(&view, &options));
    assert!(matches!(
        validate(&image("/a.png")),
        Err(DomPolicyError::UnsupportedTag(_))
    ));
    assert!(validate_with_options(&image("/a.png"), &navigation()).is_err());
    assert!(
        validate_with_options(
            &Element::new("a")
                .attribute("href", "https://docs.example.invalid/")
                .into(),
            &changed
        )
        .is_err()
    );
}

#[test]
fn yt221_selected_links_and_provisional_native_boundary() {
    let opts = navigation();
    for href in [
        "/docs",
        "#heading",
        "https://docs.example.invalid/guide?q=a%20b",
        "https://unlisted.invalid/",
    ] {
        // Native spelling alone never proves membership, even for an unlisted origin.
        assert_eq!(
            validate_with_options(&Element::new("a").attribute("href", href).into(), &opts),
            Ok(())
        );
    }
    for href in [
        "//docs.example.invalid/",
        "https:/docs.example.invalid",
        "https://",
        "http://docs.example.invalid/",
        "https://docs.example.invalid/\n",
        "https://docs.example.invalid/ a",
        "https://docs.example.invalid/\\a",
        "javascript:alert(1)",
        "data:text/html,x",
        "blob:https://docs.example.invalid/x",
    ] {
        assert!(
            validate_with_options(&Element::new("a").attribute("href", href).into(), &opts)
                .is_err(),
            "{href}"
        );
    }
    let invalid_policy = DomOptions::default()
        .with_navigation_policy(NavigationPolicy::HttpsOrigins(vec!["*".into()]));
    assert_eq!(
        validate_with_options(&Element::new("p").into(), &invalid_policy),
        Ok(())
    );
    // Browser preflight rejects this configuration; native API promises no parser proof.
}

#[test]
fn yt221_noopener_and_finite_anchor_attributes() {
    let opts = navigation();
    let link = || {
        Element::new("a")
            .attribute("href", "/docs")
            .attribute("target", "_blank")
    };
    assert_eq!(
        validate_with_options(&link().into(), &opts),
        Err(DomPolicyError::InvalidAttributeValue("rel".into()))
    );
    for rel in [
        "noopener",
        "NOOPENER noreferrer nofollow external",
        "noopener\tnoopener",
    ] {
        assert_eq!(
            validate_with_options(&link().attribute("rel", rel).into(), &opts),
            Ok(())
        );
    }
    for rel in [
        "noreferrer",
        "noopener opener",
        "preload",
        "dns-prefetch",
        "noopener\u{a0}noreferrer",
    ] {
        assert!(validate_with_options(&link().attribute("rel", rel).into(), &opts).is_err());
    }
    for (name, value) in [
        ("target", "named-window"),
        ("referrerpolicy", "unsafe-url"),
        ("ping", "/audit"),
        ("download", "a.txt"),
    ] {
        assert!(
            validate_with_options(&Element::new("a").attribute(name, value).into(), &opts).is_err()
        );
    }
    assert!(validate(&link().attribute("rel", "noopener").into()).is_err());
    let empty_list =
        DomOptions::default().with_navigation_policy(NavigationPolicy::HttpsOrigins(Vec::new()));
    assert_eq!(
        validate_with_options(&link().attribute("rel", "noopener").into(), &empty_list),
        Ok(())
    );
}

#[test]
fn yt221_images_require_explicit_selection_and_scoped_values() {
    let opts = DomOptions::default().with_image_policy(ImagePolicy::SameOrigin);
    assert_eq!(validate_with_options(&image("/a.png"), &opts), Ok(()));
    for view in [
        Element::new("img").attribute("src", "/a.png"),
        Element::new("img").attribute("alt", ""),
        Element::new("img")
            .attribute("src", "")
            .attribute("alt", ""),
    ] {
        assert!(validate_with_options(&view.into(), &opts).is_err());
    }
    let view = Element::new("img")
        .attribute("src", "/a.png")
        .attribute("alt", "")
        .child(View::text("invalid"));
    assert!(matches!(
        validate_with_options(&view.into(), &opts),
        Err(DomPolicyError::VoidElementHasChildren(_))
    ));
    for (name, value) in [
        ("loading", "eager"),
        ("decoding", "async"),
        ("fetchpriority", "high"),
        ("width", "0"),
        ("height", "4294967295"),
        ("referrerpolicy", "same-origin"),
    ] {
        let view = Element::new("img")
            .attribute("src", "/a.png")
            .attribute("alt", "")
            .attribute(name, value);
        assert_eq!(validate_with_options(&view.into(), &opts), Ok(()));
    }
    for (name, value) in [
        ("srcset", "/a.png 2x"),
        ("sizes", "100vw"),
        ("crossorigin", "anonymous"),
        ("usemap", "#map"),
        ("onerror", "evil()"),
        ("width", "-1"),
        ("loading", "auto"),
        ("decoding", "bad"),
        ("referrerpolicy", "unsafe-url"),
    ] {
        let view = Element::new("img")
            .attribute("src", "/a.png")
            .attribute("alt", "")
            .attribute(name, value);
        assert!(
            validate_with_options(&view.into(), &opts).is_err(),
            "{name}"
        );
    }
    for src in [
        "#image",
        "relative.png",
        "//other.invalid/x",
        "https://images.invalid/x",
        "data:image/png,x",
    ] {
        assert!(validate_with_options(&image(src), &opts).is_err());
    }
    let external = opts.with_image_policy(ImagePolicy::HttpsOrigins(vec![
        "https://images.invalid".into(),
    ]));
    assert_eq!(
        validate_with_options(&image("https://images.invalid/x"), &external),
        Ok(())
    );
}

#[test]
fn yt221_forms_and_other_tags_do_not_inherit_url_permissions() {
    let opts = navigation().with_image_policy(ImagePolicy::HttpsOrigins(vec![
        "https://images.invalid".into(),
    ]));
    for view in [
        Element::new("form").attribute("action", "https://docs.example.invalid/"),
        Element::new("button").attribute("formaction", "/post"),
        Element::new("form").attribute("target", "_blank"),
        Element::new("input").attribute("src", "/image.png"),
        Element::new("div").attribute("href", "/x"),
        Element::new("picture"),
        Element::new("source"),
        Element::new("svg"),
        Element::new("script"),
    ] {
        assert!(validate_with_options(&view.into(), &opts).is_err());
    }
    assert_eq!(
        validate_with_options(
            &Element::new("form")
                .attribute("method", "post")
                .attribute("action", "/post")
                .into(),
            &opts
        ),
        Ok(())
    );
    let late = Element::new("section")
        .child(image("/must-not-fetch.png"))
        .child(Element::new("div").attribute("onclick", "bad()"));
    assert!(validate_with_options(&late.into(), &opts).is_err());
}
