//! Minimal complete-document metadata owned by the native server adapter.
//!
//! The contract is deliberately smaller than a general head manager: an
//! escaped title plus a validated root language. Charset, viewport and CSP
//! remain framework-owned correctness/security policy.

use std::{error::Error, fmt};

const DEFAULT_LANGUAGE: &str = "zh-Hant";
const MAX_LANGUAGE_TAG_LEN: usize = 35;

/// A conservative, serialization-safe HTML language tag.
///
/// This validates the common ASCII shape needed by Yetlet applications. It is
/// not a complete BCP 47 registry implementation and performs no casing or
/// registry canonicalization.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LanguageTag(String);

/// Returned when a root language is outside Yetlet's bounded ASCII contract.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LanguageTagError;

impl fmt::Display for LanguageTagError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(
            "language tag must contain bounded ASCII alphanumeric subtags separated by hyphens",
        )
    }
}

impl Error for LanguageTagError {}

impl LanguageTag {
    /// Validate a language tag such as `en`, `en-US` or `zh-Hant`.
    pub fn new(value: impl Into<String>) -> Result<Self, LanguageTagError> {
        let value = value.into();
        if !valid_language_tag(&value) {
            return Err(LanguageTagError);
        }
        Ok(Self(value))
    }

    /// Return the validated tag exactly as supplied by the application.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn default_document_language() -> Self {
        Self(DEFAULT_LANGUAGE.to_owned())
    }
}

/// The bounded metadata required to render one complete HTML document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentMetadata {
    title: String,
    language: LanguageTag,
}

impl DocumentMetadata {
    /// Create metadata with the legacy `zh-Hant` document language.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            language: LanguageTag::default_document_language(),
        }
    }

    /// Create metadata with an explicitly validated root language.
    pub fn with_language(title: impl Into<String>, language: LanguageTag) -> Self {
        Self {
            title: title.into(),
            language,
        }
    }

    /// Return the unescaped title; rendering applies the existing HTML rules.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Return the validated root language.
    pub fn language(&self) -> &LanguageTag {
        &self.language
    }
}

fn valid_language_tag(value: &str) -> bool {
    if value.is_empty() || value.len() > MAX_LANGUAGE_TAG_LEN || !value.is_ascii() {
        return false;
    }
    let mut parts = value.split('-');
    let Some(primary) = parts.next() else {
        return false;
    };
    if primary.is_empty()
        || primary.len() > 8
        || !primary.bytes().all(|byte| byte.is_ascii_alphabetic())
    {
        return false;
    }
    parts.all(|part| {
        !part.is_empty() && part.len() <= 8 && part.bytes().all(|byte| byte.is_ascii_alphanumeric())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn common_language_tags_are_preserved() {
        for tag in ["en", "en-US", "zh-Hant", "de-1996", "x-private"] {
            assert_eq!(LanguageTag::new(tag).unwrap().as_str(), tag);
        }
    }

    #[test]
    fn unsafe_or_ambiguous_language_tags_fail_closed() {
        for tag in [
            "",
            "-en",
            "en-",
            "en--US",
            "en_US",
            "en<script>",
            " en",
            "é",
            "abcdefghi",
            "en-123456789",
        ] {
            assert_eq!(LanguageTag::new(tag), Err(LanguageTagError), "{tag}");
        }
    }

    #[test]
    fn metadata_defaults_to_the_legacy_language() {
        let metadata = DocumentMetadata::new("Title");
        assert_eq!(metadata.title(), "Title");
        assert_eq!(metadata.language().as_str(), "zh-Hant");
    }
}
