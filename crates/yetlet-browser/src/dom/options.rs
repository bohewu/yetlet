//! Public policy values; browser URL preflight is the enforcement boundary.
use super::{DomPolicyError, navigation_spelling};
use yetlet::view::Element;

/// Additional anchor navigation capability. Origins are public configuration.
///
/// Native validation does not parse these strings or prove membership. Mounting
/// requires canonical HTTPS origins (an optional trailing slash is accepted),
/// with no credentials, path, query, fragment or wildcard.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum NavigationPolicy {
    /// Preserve the original root-relative/fragment and same-origin policy.
    #[default]
    SameOrigin,
    /// Keep same-origin navigation and add these named HTTPS origins.
    /// Also explicitly selects finite target/rel/link-metadata attributes.
    HttpsOrigins(Vec<String>),
}

/// Image loading is separate from navigation and is disabled by default.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ImagePolicy {
    /// Reject img, including local images.
    #[default]
    Disabled,
    /// Admit only root-relative image URLs resolving to the document origin.
    SameOrigin,
    /// Admit same-origin paths and absolute HTTPS at these additional origins.
    /// Origin configuration is enforced by browser preflight, not construction.
    HttpsOrigins(Vec<String>),
}

/// Explicit, per-mount DOM capabilities. No global configuration is read.
///
/// Builders store values without origin validation. Both legacy entry points
/// use these defaults. Native `validate_with_options` remains provisional;
/// browser mounting checks all policy origins before creating any nodes.
///
/// ```
/// use yetlet_browser::dom::{DomOptions, ImagePolicy, NavigationPolicy};
/// let options = DomOptions::default()
///     .with_navigation_policy(NavigationPolicy::HttpsOrigins(vec![
///         "https://docs.example.invalid".into(),
///     ]))
///     .with_image_policy(ImagePolicy::SameOrigin);
/// assert_eq!(options.image_policy(), &ImagePolicy::SameOrigin);
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DomOptions {
    navigation_policy: NavigationPolicy,
    image_policy: ImagePolicy,
}

impl DomOptions {
    pub fn navigation_policy(&self) -> &NavigationPolicy {
        &self.navigation_policy
    }
    pub fn image_policy(&self) -> &ImagePolicy {
        &self.image_policy
    }
    pub fn with_navigation_policy(mut self, policy: NavigationPolicy) -> Self {
        self.navigation_policy = policy;
        self
    }
    pub fn with_image_policy(mut self, policy: ImagePolicy) -> Self {
        self.image_policy = policy;
        self
    }

    pub(super) fn images(&self) -> bool {
        self.image_policy != ImagePolicy::Disabled
    }

    pub(super) fn extended_links(&self) -> bool {
        matches!(self.navigation_policy, NavigationPolicy::HttpsOrigins(_))
    }

    pub(super) fn attribute(&self, tag: &str, name: &str) -> bool {
        (tag == "a"
            && self.extended_links()
            && matches!(name, "target" | "rel" | "hreflang" | "referrerpolicy"))
            || (tag == "img"
                && self.images()
                && matches!(
                    name,
                    "src"
                        | "alt"
                        | "width"
                        | "height"
                        | "loading"
                        | "decoding"
                        | "fetchpriority"
                        | "referrerpolicy"
                ))
    }

    pub(super) fn url_spelling(&self, name: &str, value: &str) -> bool {
        match name {
            "action" => navigation_spelling(value, false),
            "href" => {
                navigation_spelling(value, true) || (self.extended_links() && https_spelling(value))
            }
            "src" => {
                navigation_spelling(value, false)
                    || (matches!(self.image_policy, ImagePolicy::HttpsOrigins(_))
                        && https_spelling(value))
            }
            _ => true,
        }
    }
}

fn https_spelling(value: &str) -> bool {
    // Provisional spelling, deliberately not an origin or URL parser.
    value
        .strip_prefix("https://")
        .is_some_and(|tail| !tail.is_empty())
        && !value
            .chars()
            .any(|c| c.is_control() || c.is_whitespace() || c == '\\')
}

pub(super) fn attribute_value(name: &str, value: &str) -> bool {
    let keyword = |items: &[&str]| items.iter().any(|item| value.eq_ignore_ascii_case(item));
    match name {
        "target" => matches!(value, "_self" | "_blank"),
        "rel" => value.split_ascii_whitespace().all(|token| {
            ["noopener", "noreferrer", "nofollow", "external"]
                .iter()
                .any(|s| token.eq_ignore_ascii_case(s))
        }),
        "referrerpolicy" => keyword(&["no-referrer", "same-origin"]),
        "loading" => keyword(&["eager", "lazy"]),
        "decoding" => keyword(&["auto", "sync", "async"]),
        "fetchpriority" => keyword(&["auto", "high", "low"]),
        _ => true,
    }
}

pub(super) fn element_values(element: &Element) -> Result<(), DomPolicyError> {
    let attr = |name| {
        element
            .attributes()
            .iter()
            .find(|a| a.name == name)
            .map(|a| a.value.as_str())
    };
    if element.tag() == "img" {
        for name in ["src", "alt"] {
            if attr(name).is_none() {
                return Err(DomPolicyError::InvalidAttributeValue(name.into()));
            }
        }
    }
    if element.tag() == "a"
        && attr("target") == Some("_blank")
        && !attr("rel").is_some_and(|s| {
            s.split_ascii_whitespace()
                .any(|token| token.eq_ignore_ascii_case("noopener"))
        })
    {
        return Err(DomPolicyError::InvalidAttributeValue("rel".into()));
    }
    Ok(())
}
