//! Renderer-neutral identity, key, and slot contracts for bounded DOM bindings.
//!
//! These types validate the vocabulary shared by native SSR and browser
//! bindings. They describe identity and safe inert update slots; they do not
//! select a renderer, template language, scheduler, or hydration protocol.

use std::fmt;

/// Validation failures for renderer-neutral binding metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BindingContractError {
    InvalidComponentName,
    InvalidInstanceName,
    InvalidSlotName,
    InvalidKey(String),
    UnsupportedAttribute(String),
}

impl fmt::Display for BindingContractError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidComponentName => formatter.write_str("invalid component name"),
            Self::InvalidInstanceName => formatter.write_str("invalid component instance"),
            Self::InvalidSlotName => formatter.write_str("invalid binding slot name"),
            Self::InvalidKey(key) => write!(formatter, "invalid binding key: {key:?}"),
            Self::UnsupportedAttribute(name) => {
                write!(formatter, "unsupported inert binding attribute: {name}")
            }
        }
    }
}

impl std::error::Error for BindingContractError {}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Token(String);

impl Token {
    fn parse(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        let mut bytes = value.bytes();
        let valid = matches!(bytes.next(), Some(b'a'..=b'z'))
            && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
        valid.then_some(Self(value))
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

/// Stable component and instance identity shared by SSR and browser binding.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ComponentIdentity {
    component: Token,
    instance: Token,
}

impl ComponentIdentity {
    pub fn new(
        component: impl Into<String>,
        instance: impl Into<String>,
    ) -> Result<Self, BindingContractError> {
        let component =
            Token::parse(component).ok_or(BindingContractError::InvalidComponentName)?;
        let instance = Token::parse(instance).ok_or(BindingContractError::InvalidInstanceName)?;
        Ok(Self {
            component,
            instance,
        })
    }

    pub fn component(&self) -> &str {
        self.component.as_str()
    }

    pub fn instance(&self) -> &str {
        self.instance.as_str()
    }
}

/// Stable semantic key. The value is later escaped by the selected renderer.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BindingKey(String);

impl BindingKey {
    pub fn new(value: impl Into<String>) -> Result<Self, BindingContractError> {
        let value = value.into();
        if value.is_empty() || value.chars().any(char::is_control) {
            return Err(BindingContractError::InvalidKey(value));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for BindingKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A validated dynamic slot name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SlotName(Token);

impl SlotName {
    pub fn new(value: impl Into<String>) -> Result<Self, BindingContractError> {
        Token::parse(value)
            .map(Self)
            .ok_or(BindingContractError::InvalidSlotName)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl fmt::Display for SlotName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// An allowlisted inert attribute name suitable for a dynamic slot.
///
/// Executable and browser-parsing-sensitive attributes are excluded. Yetlet's
/// reserved `data-yetlet-*` namespace cannot be shadowed by user descriptors.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InertAttributeName(String);

impl InertAttributeName {
    pub fn new(value: impl Into<String>) -> Result<Self, BindingContractError> {
        let value = value.into();
        let valid_name = !value.is_empty()
            && value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
        let allowlisted = matches!(value.as_str(), "class" | "title")
            || value
                .strip_prefix("data-")
                .is_some_and(|suffix| !suffix.is_empty())
            || value
                .strip_prefix("aria-")
                .is_some_and(|suffix| !suffix.is_empty());
        if !valid_name
            || !allowlisted
            || value.starts_with("data-yetlet-")
            || value.starts_with("on")
            || matches!(value.as_str(), "style" | "srcdoc")
        {
            return Err(BindingContractError::UnsupportedAttribute(value));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for InertAttributeName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// Renderer-neutral declaration of a dynamic text or inert-attribute slot.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SlotDescriptor {
    Text {
        slot: SlotName,
    },
    Attribute {
        slot: SlotName,
        attribute: InertAttributeName,
    },
}

impl SlotDescriptor {
    pub fn text(slot: impl Into<String>) -> Result<Self, BindingContractError> {
        Ok(Self::Text {
            slot: SlotName::new(slot)?,
        })
    }

    pub fn attribute(
        slot: impl Into<String>,
        attribute: impl Into<String>,
    ) -> Result<Self, BindingContractError> {
        Ok(Self::Attribute {
            slot: SlotName::new(slot)?,
            attribute: InertAttributeName::new(attribute)?,
        })
    }

    pub fn slot(&self) -> &SlotName {
        match self {
            Self::Text { slot } | Self::Attribute { slot, .. } => slot,
        }
    }

    pub fn attribute_name(&self) -> Option<&InertAttributeName> {
        match self {
            Self::Text { .. } => None,
            Self::Attribute { attribute, .. } => Some(attribute),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BindingContractError, BindingKey, ComponentIdentity, InertAttributeName, SlotDescriptor,
        SlotName,
    };

    #[test]
    fn component_identity_uses_a_deterministic_lowercase_token_contract() {
        let identity = ComponentIdentity::new("todo-list", "instance-1").unwrap();
        assert_eq!(identity.component(), "todo-list");
        assert_eq!(identity.instance(), "instance-1");
        assert_eq!(
            ComponentIdentity::new("TodoList", "instance-1"),
            Err(BindingContractError::InvalidComponentName)
        );
        assert_eq!(
            ComponentIdentity::new("todo-list", "1-instance"),
            Err(BindingContractError::InvalidInstanceName)
        );
    }

    #[test]
    fn keys_reject_empty_or_control_character_values() {
        assert_eq!(BindingKey::new("42").unwrap().as_str(), "42");
        assert_eq!(BindingKey::new("row-é").unwrap().as_str(), "row-é");
        assert_eq!(
            BindingKey::new(""),
            Err(BindingContractError::InvalidKey(String::new()))
        );
        assert!(matches!(
            BindingKey::new("bad\nkey"),
            Err(BindingContractError::InvalidKey(_))
        ));
    }

    #[test]
    fn slot_names_and_descriptors_are_typed() {
        let text = SlotDescriptor::text("label").unwrap();
        assert_eq!(text.slot().as_str(), "label");
        assert_eq!(text.attribute_name(), None);

        let attribute = SlotDescriptor::attribute("state", "aria-pressed").unwrap();
        assert_eq!(attribute.slot().as_str(), "state");
        assert_eq!(attribute.attribute_name().unwrap().as_str(), "aria-pressed");
        assert_eq!(
            SlotName::new("BadSlot"),
            Err(BindingContractError::InvalidSlotName)
        );
    }

    #[test]
    fn inert_attributes_preserve_the_reserved_and_executable_boundary() {
        for allowed in ["class", "title", "data-state", "aria-selected"] {
            assert_eq!(InertAttributeName::new(allowed).unwrap().as_str(), allowed);
        }
        for rejected in [
            "onclick",
            "style",
            "srcdoc",
            "data-yetlet-key",
            "data-",
            "aria-",
            "DATA-state",
        ] {
            assert!(matches!(
                InertAttributeName::new(rejected),
                Err(BindingContractError::UnsupportedAttribute(_))
            ));
        }
    }
}
