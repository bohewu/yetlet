//! Isolated M7b keyed/static-slot authoring prototype.
//!
//! This crate is deliberately outside Yetlet's production renderer crates. It
//! explores a small API for one static keyed-list shape with typed dynamic text,
//! inert attributes, one conditional text node and one local row action. It is
//! not a general VDOM, signal runtime, macro DSL or stable public API.

use std::{collections::HashSet, fmt, rc::Rc};

pub use yetlet_core::binding::ComponentIdentity;
use yetlet_core::{
    Element as ViewElement, View,
    binding::{BindingContractError, BindingKey, InertAttributeName, SlotName},
};

pub mod reactive;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateError {
    InvalidIdentity(&'static str),
    InvalidSlot(&'static str),
    InvalidAction(&'static str),
    InvalidInput(&'static str),
    UnsupportedAttribute(String),
    DuplicateAttribute(String),
    InvalidKey(String),
    DuplicateKey(String),
}

impl fmt::Display for TemplateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidIdentity(field) => write!(formatter, "invalid component {field}"),
            Self::InvalidSlot(field) => write!(formatter, "invalid slot {field}"),
            Self::InvalidAction(field) => write!(formatter, "invalid row action {field}"),
            Self::InvalidInput(field) => write!(formatter, "invalid row input {field}"),
            Self::UnsupportedAttribute(name) => {
                write!(formatter, "unsupported dynamic attribute: {name}")
            }
            Self::DuplicateAttribute(name) => write!(formatter, "duplicate attribute: {name}"),
            Self::InvalidKey(key) => write!(formatter, "invalid keyed item: {key:?}"),
            Self::DuplicateKey(key) => write!(formatter, "duplicate keyed item: {key}"),
        }
    }
}

impl std::error::Error for TemplateError {}

impl From<BindingContractError> for TemplateError {
    fn from(error: BindingContractError) -> Self {
        match error {
            BindingContractError::InvalidComponentName => Self::InvalidIdentity("name"),
            BindingContractError::InvalidInstanceName => Self::InvalidIdentity("instance"),
            BindingContractError::InvalidSlotName => Self::InvalidSlot("name"),
            BindingContractError::InvalidKey(key) => Self::InvalidKey(key),
            BindingContractError::UnsupportedAttribute(name) => Self::UnsupportedAttribute(name),
        }
    }
}

type OptionalStringSlot<T> = Rc<dyn Fn(&T) -> Option<String>>;

struct DynamicAttribute<T> {
    name: &'static str,
    value: OptionalStringSlot<T>,
}

impl<T> Clone for DynamicAttribute<T> {
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            value: Rc::clone(&self.value),
        }
    }
}

struct ConditionalText<T> {
    slot: &'static str,
    class: Option<&'static str>,
    value: OptionalStringSlot<T>,
}

impl<T> Clone for ConditionalText<T> {
    fn clone(&self) -> Self {
        Self {
            slot: self.slot,
            class: self.class,
            value: Rc::clone(&self.value),
        }
    }
}

struct RowInput<T, K, Message> {
    slot: &'static str,
    aria_label: String,
    value: Rc<dyn Fn(&T) -> String>,
    message: Rc<dyn Fn(K, String) -> Message>,
}

impl<T, K, Message> Clone for RowInput<T, K, Message> {
    fn clone(&self) -> Self {
        Self {
            slot: self.slot,
            aria_label: self.aria_label.clone(),
            value: Rc::clone(&self.value),
            message: Rc::clone(&self.message),
        }
    }
}

struct RowAction<K, Message> {
    label: String,
    message: Rc<dyn Fn(K) -> Message>,
}

impl<K, Message> Clone for RowAction<K, Message> {
    fn clone(&self) -> Self {
        Self {
            label: self.label.clone(),
            message: Rc::clone(&self.message),
        }
    }
}

/// Bounded keyed-list template used by the M7b prototype.
///
/// The shape is intentionally fixed: `ul > li > label span + optional text
/// input + optional conditional span + optional button`. Static structure is
/// attached once; updates only touch declared slots. General trees belong to
/// later research.
///
/// ```
/// use yetlet_keyed_bindings::{ComponentIdentity, KeyedListTemplate, TemplateError};
///
/// #[derive(Clone)]
/// struct Item { id: u64, label: String, draft: String, selected: bool }
/// #[derive(Clone)]
/// enum Message { Select(u64), Edit(u64, String) }
///
/// # fn example() -> Result<(), TemplateError> {
/// let template = KeyedListTemplate::new(
///     ComponentIdentity::new("item-list", "example")?,
///     |item: &Item| item.id,
///     |item: &Item| item.label.clone(),
/// )
/// .dynamic_attribute("aria-selected", |item| Some(item.selected.to_string()))?
/// .text_input("draft", "Edit item", |item| item.draft.clone(), Message::Edit)?
/// .conditional_text("status", None, |item| item.selected.then(|| "Selected".into()))?
/// .on_row_click("Select", Message::Select)?;
/// let items = vec![Item {
///     id: 1, label: "One".into(), draft: "Draft".into(), selected: false,
/// }];
/// let _ssr_view = template.ssr_view(&items)?;
/// # Ok(()) }
/// # example().unwrap();
/// ```
pub struct KeyedListTemplate<T, K, Message> {
    identity: ComponentIdentity,
    key: Rc<dyn Fn(&T) -> K>,
    label: Rc<dyn Fn(&T) -> String>,
    list_attributes: Vec<(&'static str, String)>,
    row_attributes: Vec<(&'static str, String)>,
    dynamic_attributes: Vec<DynamicAttribute<T>>,
    input: Option<RowInput<T, K, Message>>,
    conditional: Option<ConditionalText<T>>,
    action: Option<RowAction<K, Message>>,
}

impl<T, K, Message> Clone for KeyedListTemplate<T, K, Message> {
    fn clone(&self) -> Self {
        Self {
            identity: self.identity.clone(),
            key: Rc::clone(&self.key),
            label: Rc::clone(&self.label),
            list_attributes: self.list_attributes.clone(),
            row_attributes: self.row_attributes.clone(),
            dynamic_attributes: self.dynamic_attributes.clone(),
            input: self.input.clone(),
            conditional: self.conditional.clone(),
            action: self.action.clone(),
        }
    }
}

impl<T, K, Message> KeyedListTemplate<T, K, Message> {
    pub fn new(
        identity: ComponentIdentity,
        key: impl Fn(&T) -> K + 'static,
        label: impl Fn(&T) -> String + 'static,
    ) -> Self {
        Self {
            identity,
            key: Rc::new(key),
            label: Rc::new(label),
            list_attributes: Vec::new(),
            row_attributes: Vec::new(),
            dynamic_attributes: Vec::new(),
            input: None,
            conditional: None,
            action: None,
        }
    }

    pub fn list_attribute(
        mut self,
        name: &'static str,
        value: impl Into<String>,
    ) -> Result<Self, TemplateError> {
        validate_attribute(name)?;
        if self
            .list_attributes
            .iter()
            .any(|(current, _)| *current == name)
        {
            return Err(TemplateError::DuplicateAttribute(name.to_owned()));
        }
        self.list_attributes.push((name, value.into()));
        Ok(self)
    }

    pub fn row_attribute(
        mut self,
        name: &'static str,
        value: impl Into<String>,
    ) -> Result<Self, TemplateError> {
        validate_attribute(name)?;
        if self
            .row_attributes
            .iter()
            .any(|(current, _)| *current == name)
            || self
                .dynamic_attributes
                .iter()
                .any(|attribute| attribute.name == name)
        {
            return Err(TemplateError::DuplicateAttribute(name.to_owned()));
        }
        self.row_attributes.push((name, value.into()));
        Ok(self)
    }

    pub fn dynamic_attribute(
        mut self,
        name: &'static str,
        value: impl Fn(&T) -> Option<String> + 'static,
    ) -> Result<Self, TemplateError> {
        validate_attribute(name)?;
        if self
            .row_attributes
            .iter()
            .any(|(current, _)| *current == name)
            || self
                .dynamic_attributes
                .iter()
                .any(|attribute| attribute.name == name)
        {
            return Err(TemplateError::DuplicateAttribute(name.to_owned()));
        }
        self.dynamic_attributes.push(DynamicAttribute {
            name,
            value: Rc::new(value),
        });
        Ok(self)
    }

    pub fn conditional_text(
        mut self,
        slot: &'static str,
        class: Option<&'static str>,
        value: impl Fn(&T) -> Option<String> + 'static,
    ) -> Result<Self, TemplateError> {
        if SlotName::new(slot).is_err()
            || slot == "label"
            || self.input.as_ref().is_some_and(|input| input.slot == slot)
        {
            return Err(TemplateError::InvalidSlot("name"));
        }
        if self.conditional.is_some() {
            return Err(TemplateError::InvalidSlot("duplicate"));
        }
        if let Some(class_name) = class
            && (class_name.is_empty() || class_name.chars().any(char::is_control))
        {
            return Err(TemplateError::InvalidSlot("class"));
        }
        self.conditional = Some(ConditionalText {
            slot,
            class,
            value: Rc::new(value),
        });
        Ok(self)
    }

    pub fn text_input(
        mut self,
        slot: &'static str,
        aria_label: impl Into<String>,
        value: impl Fn(&T) -> String + 'static,
        message: impl Fn(K, String) -> Message + 'static,
    ) -> Result<Self, TemplateError> {
        let aria_label = aria_label.into();
        if SlotName::new(slot).is_err()
            || slot == "label"
            || self
                .conditional
                .as_ref()
                .is_some_and(|conditional| conditional.slot == slot)
        {
            return Err(TemplateError::InvalidInput("slot"));
        }
        if aria_label.trim().is_empty() || aria_label.chars().any(char::is_control) {
            return Err(TemplateError::InvalidInput("aria-label"));
        }
        if self.input.is_some() {
            return Err(TemplateError::InvalidInput("duplicate"));
        }
        self.input = Some(RowInput {
            slot,
            aria_label,
            value: Rc::new(value),
            message: Rc::new(message),
        });
        Ok(self)
    }

    pub fn on_row_click(
        mut self,
        label: impl Into<String>,
        message: impl Fn(K) -> Message + 'static,
    ) -> Result<Self, TemplateError> {
        let label = label.into();
        if label.trim().is_empty() || label.chars().any(char::is_control) {
            return Err(TemplateError::InvalidAction("label"));
        }
        if self.action.is_some() {
            return Err(TemplateError::InvalidAction("duplicate"));
        }
        self.action = Some(RowAction {
            label,
            message: Rc::new(message),
        });
        Ok(self)
    }

    pub fn identity(&self) -> &ComponentIdentity {
        &self.identity
    }

    pub fn ssr_view(&self, items: &[T]) -> Result<View, TemplateError>
    where
        K: ToString,
    {
        validate_keys(items, &self.key)?;
        let mut list = ViewElement::new("ul")
            .attribute("data-yetlet-component", self.identity.component())
            .attribute("data-yetlet-instance", self.identity.instance());
        for (name, value) in &self.list_attributes {
            list = list.attribute(name, value);
        }
        let rows = items
            .iter()
            .map(|item| self.ssr_row(item))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(list.children(rows).into())
    }

    fn ssr_row(&self, item: &T) -> Result<View, TemplateError>
    where
        K: ToString,
    {
        let key = (self.key)(item).to_string();
        let mut row = ViewElement::new("li").attribute("data-yetlet-key", key);
        for (name, value) in &self.row_attributes {
            row = row.attribute(name, value);
        }
        for attribute in &self.dynamic_attributes {
            if let Some(value) = (attribute.value)(item) {
                row = row.attribute(attribute.name, value);
            }
        }
        row = row.child(
            ViewElement::new("span")
                .attribute("data-yetlet-slot", "label")
                .child(View::text((self.label)(item))),
        );
        if let Some(input) = &self.input {
            row = row.child(
                ViewElement::new("input")
                    .attribute("type", "text")
                    .attribute("data-yetlet-input", input.slot)
                    .attribute("aria-label", &input.aria_label)
                    .attribute("value", (input.value)(item)),
            );
        }
        if let Some(conditional) = &self.conditional
            && let Some(value) = (conditional.value)(item)
        {
            let mut node = ViewElement::new("span")
                .attribute("data-yetlet-slot", conditional.slot)
                .child(View::text(value));
            if let Some(class_name) = conditional.class {
                node = node.attribute("class", class_name);
            }
            row = row.child(node);
        }
        if let Some(action) = &self.action {
            row = row.child(
                ViewElement::new("button")
                    .attribute("type", "button")
                    .attribute("data-yetlet-action", "row")
                    .child(View::text(&action.label)),
            );
        }
        Ok(row.into())
    }
}

fn validate_attribute(name: &str) -> Result<(), TemplateError> {
    InertAttributeName::new(name)
        .map(|_| ())
        .map_err(TemplateError::from)
}

fn validate_keys<T, K>(items: &[T], key: &Rc<dyn Fn(&T) -> K>) -> Result<(), TemplateError>
where
    K: ToString,
{
    let mut keys = HashSet::with_capacity(items.len());
    for item in items {
        let value = BindingKey::new(key(item).to_string())
            .map_err(TemplateError::from)?
            .into_string();
        if !keys.insert(value.clone()) {
            return Err(TemplateError::DuplicateKey(value));
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemoItem {
    pub id: u64,
    pub title: String,
    pub draft: String,
    pub completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DemoMessage {
    Toggle(u64),
    Edit(u64, String),
    Reverse,
    Append,
    RemoveFirst,
    Reset,
    Reject,
    InjectDuplicate,
}

#[cfg(target_arch = "wasm32")]
fn reduce_demo(items: &mut Vec<DemoItem>, message: DemoMessage) -> Result<(), String> {
    match message {
        DemoMessage::Toggle(id) => {
            let item = items
                .iter_mut()
                .find(|item| item.id == id)
                .ok_or_else(|| format!("missing demo item: {id}"))?;
            item.completed = !item.completed;
        }
        DemoMessage::Edit(id, value) => {
            let item = items
                .iter_mut()
                .find(|item| item.id == id)
                .ok_or_else(|| format!("missing demo item: {id}"))?;
            item.draft = value;
        }
        DemoMessage::Reverse => items.reverse(),
        DemoMessage::Append => {
            let next = items
                .iter()
                .map(|item| item.id)
                .max()
                .unwrap_or(0)
                .checked_add(1)
                .ok_or_else(|| "demo item ID overflow".to_owned())?;
            items.push(DemoItem {
                id: next,
                title: format!("Appended item {next}"),
                draft: format!("Draft {next}"),
                completed: false,
            });
        }
        DemoMessage::RemoveFirst => {
            let index = items
                .iter()
                .position(|item| item.id == 1)
                .ok_or_else(|| "demo item 1 is already removed".to_owned())?;
            items.remove(index);
        }
        DemoMessage::Reset => *items = demo_items(),
        DemoMessage::Reject => return Err("rejected local demo message".to_owned()),
        DemoMessage::InjectDuplicate => {
            if items.len() < 2 {
                return Err("duplicate-key fixture requires two items".to_owned());
            }
            items[1].id = items[0].id;
        }
    }
    Ok(())
}

pub fn demo_items() -> Vec<DemoItem> {
    vec![
        DemoItem {
            id: 1,
            title: "Preserve SSR identity".into(),
            draft: "Draft one".into(),
            completed: false,
        },
        DemoItem {
            id: 2,
            title: "Update typed slots locally".into(),
            draft: "Draft two".into(),
            completed: false,
        },
        DemoItem {
            id: 3,
            title: "Keep keyed rows stable".into(),
            draft: "Draft three".into(),
            completed: true,
        },
    ]
}

pub fn demo_template() -> KeyedListTemplate<DemoItem, u64, DemoMessage> {
    KeyedListTemplate::new(
        ComponentIdentity::new("todo-list", "m7b-demo").expect("static identity"),
        |item: &DemoItem| item.id,
        |item: &DemoItem| {
            format!(
                "{}: {}",
                item.title,
                if item.completed { "complete" } else { "active" }
            )
        },
    )
    .list_attribute("class", "m7b-list")
    .expect("static list attribute")
    .row_attribute("class", "m7b-row")
    .expect("static row attribute")
    .dynamic_attribute("data-completed", |item| Some(item.completed.to_string()))
    .expect("static dynamic attribute")
    .dynamic_attribute("aria-pressed", |item| Some(item.completed.to_string()))
    .expect("static dynamic attribute")
    .dynamic_attribute("data-draft", |item| Some(item.draft.clone()))
    .expect("static draft attribute")
    .text_input(
        "draft",
        "Edit item draft",
        |item| item.draft.clone(),
        DemoMessage::Edit,
    )
    .expect("static text input")
    .conditional_text("details", Some("m7b-details"), |item| {
        item.completed
            .then(|| "Completed through a local typed message".to_owned())
    })
    .expect("static conditional slot")
    .on_row_click("Toggle", DemoMessage::Toggle)
    .expect("static row action")
}

pub fn demo_ssr_view() -> Result<View, TemplateError> {
    demo_template().ssr_view(&demo_items())
}

pub fn demo_ssr_html() -> Result<String, Box<dyn std::error::Error>> {
    Ok(yetlet_html::render_to_string(&demo_ssr_view()?)?)
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use std::{
        cell::{Cell, RefCell},
        collections::{HashMap, HashSet},
        rc::Rc,
    };

    use wasm_bindgen::{JsCast, JsValue, prelude::wasm_bindgen};
    use web_sys::{Document, Element, HtmlInputElement, Node};
    use yetlet_core::binding::BindingKey;
    use yetlet_dom_keyed::{DomEventListener, KeyedChildren, KeyedEntry};

    use super::{
        DemoItem, DemoMessage, KeyedListTemplate, RowAction, RowInput, TemplateError, demo_items,
        demo_template,
        reactive::{FlushStats, GraphHandle, LocalGraph, ReactiveError, Subscription},
        reduce_demo, validate_keys,
    };

    struct PreparedRow {
        element: Element,
        label: Element,
        input: Option<HtmlInputElement>,
        input_model_value: Option<String>,
        conditional: Option<Element>,
        action: Option<Element>,
    }

    struct BoundRow {
        label: Element,
        input: Option<HtmlInputElement>,
        input_model_value: Option<String>,
        conditional: Option<Element>,
        action: Option<Element>,
        _bindings: Vec<DomEventListener>,
    }

    struct FocusedInput {
        input: HtmlInputElement,
        selection_start: Option<u32>,
        selection_end: Option<u32>,
        selection_direction: Option<String>,
    }

    fn capture_focused_input(root: &Element) -> Result<Option<FocusedInput>, JsValue> {
        let document = root
            .owner_document()
            .ok_or_else(|| JsValue::from_str("M7d root has no document"))?;
        let Some(active) = document.active_element() else {
            return Ok(None);
        };
        if !root.contains(Some(active.as_ref())) {
            return Ok(None);
        }
        let Ok(input) = active.dyn_into::<HtmlInputElement>() else {
            return Ok(None);
        };
        Ok(Some(FocusedInput {
            selection_start: input.selection_start()?,
            selection_end: input.selection_end()?,
            selection_direction: input.selection_direction()?,
            input,
        }))
    }

    fn restore_focused_input(focused: Option<FocusedInput>) -> Result<(), JsValue> {
        let Some(focused) = focused else {
            return Ok(());
        };
        if !focused.input.is_connected() {
            return Ok(());
        }
        focused.input.focus()?;
        if let (Some(start), Some(end)) = (focused.selection_start, focused.selection_end) {
            focused.input.set_selection_range_with_direction(
                start,
                end,
                focused.selection_direction.as_deref().unwrap_or("none"),
            )?;
        }
        Ok(())
    }

    pub struct BoundKeyedList<T, K, Message> {
        template: KeyedListTemplate<T, K, Message>,
        root: Element,
        rows: KeyedChildren<BoundRow>,
        dispatch: Rc<RefCell<Box<dyn FnMut(Message)>>>,
        disposed: bool,
    }

    impl<T, K, Message> BoundKeyedList<T, K, Message>
    where
        T: 'static,
        K: Clone + ToString + 'static,
        Message: 'static,
    {
        pub fn bind_existing(
            template: KeyedListTemplate<T, K, Message>,
            host: &Element,
            items: &[T],
            dispatch: impl FnMut(Message) + 'static,
        ) -> Result<Self, JsValue> {
            validate_keys(items, &template.key).map_err(template_error)?;
            let root = host
                .first_element_child()
                .ok_or_else(|| JsValue::from_str("M7b SSR root missing"))?;
            if root.get_attribute("data-yetlet-component").as_deref()
                != Some(template.identity.component())
                || root.get_attribute("data-yetlet-instance").as_deref()
                    != Some(template.identity.instance())
            {
                return Err(JsValue::from_str("M7b SSR component identity mismatch"));
            }
            if root.has_attribute("data-yetlet-bound") {
                return Err(JsValue::from_str("M7b component is already bound"));
            }
            if !root.tag_name().eq_ignore_ascii_case("ul") {
                return Err(JsValue::from_str("M7b SSR root tag mismatch"));
            }

            for (name, value) in &template.list_attributes {
                if root.get_attribute(name).as_deref() != Some(value.as_str()) {
                    return Err(JsValue::from_str("M7b SSR list attribute mismatch"));
                }
            }
            let dispatch: Rc<RefCell<Box<dyn FnMut(Message)>>> =
                Rc::new(RefCell::new(Box::new(dispatch)));
            let mut expected = HashMap::with_capacity(items.len());
            for item in items {
                expected.insert((template.key)(item).to_string(), item);
            }
            let children = direct_element_children(&root, "root")?;
            let mut seen = HashSet::with_capacity(items.len());
            let mut prepared = Vec::with_capacity(items.len());
            for element in children {
                let key_text = element
                    .get_attribute("data-yetlet-key")
                    .ok_or_else(|| JsValue::from_str("M7b SSR row key missing"))?;
                if !seen.insert(key_text.clone()) {
                    return Err(JsValue::from_str("M7b SSR contains duplicate row key"));
                }
                let item = expected
                    .remove(&key_text)
                    .ok_or_else(|| JsValue::from_str("M7b SSR contains unknown row key"))?;
                let key = (template.key)(item);
                let row = preflight_row(&template, element, item)?;
                prepared.push((key_text, row, key));
            }
            if !expected.is_empty() || prepared.len() != items.len() {
                return Err(JsValue::from_str("M7b SSR keyed row set mismatch"));
            }
            let mut rows = Vec::with_capacity(items.len());
            for (key_text, prepared, key) in prepared {
                let stable_key = binding_key(key_text)?;
                let row = bind_prepared_row(&template, prepared, key, Rc::clone(&dispatch))?;
                rows.push((stable_key, row));
            }
            let rows = KeyedChildren::from_existing(root.clone(), rows)?;
            root.set_attribute("data-yetlet-bound", "true")?;
            Ok(Self {
                template,
                root,
                rows,
                dispatch,
                disposed: false,
            })
        }

        pub fn reconcile(&mut self, items: &[T]) -> Result<(), JsValue> {
            if self.disposed {
                return Err(JsValue::from_str("M7b binding is disposed"));
            }
            let focused_input = capture_focused_input(&self.root)?;
            validate_keys(items, &self.template.key).map_err(template_error)?;
            let template = &self.template;
            let dispatch = Rc::clone(&self.dispatch);
            self.rows.reconcile(
                items,
                |item| binding_key((template.key)(item).to_string()),
                |key, element, row, item| update_row(template, key, element, row, item),
                |document, key, item| {
                    let event_key = (template.key)(item);
                    create_row(
                        template,
                        document,
                        key,
                        item,
                        event_key,
                        Rc::clone(&dispatch),
                    )
                },
            )?;
            restore_focused_input(focused_input)?;
            Ok(())
        }

        pub fn dispose(&mut self) -> Result<(), JsValue> {
            if !self.disposed {
                self.rows.dispose();
                self.root.remove_attribute("data-yetlet-bound")?;
                self.disposed = true;
            }
            Ok(())
        }
    }

    impl<T, K, Message> Drop for BoundKeyedList<T, K, Message> {
        fn drop(&mut self) {
            if !self.disposed {
                self.rows.dispose();
                let _ = self.root.remove_attribute("data-yetlet-bound");
            }
        }
    }

    fn direct_element_children(root: &Element, context: &str) -> Result<Vec<Element>, JsValue> {
        let mut elements = Vec::new();
        let mut child = root.first_child();
        while let Some(node) = child {
            child = node.next_sibling();
            if node.node_type() != Node::ELEMENT_NODE {
                return Err(JsValue::from_str(&format!(
                    "M7b SSR {context} contains an unexpected child node",
                )));
            }
            elements.push(
                node.dyn_into::<Element>()
                    .map_err(|_| JsValue::from_str("M7b SSR element conversion failed"))?,
            );
        }
        Ok(elements)
    }

    fn exact_text_node(element: &Element, expected: &str) -> bool {
        match element.first_child() {
            None => expected.is_empty(),
            Some(node) => {
                node.node_type() == Node::TEXT_NODE
                    && node.next_sibling().is_none()
                    && node.node_value().as_deref() == Some(expected)
            }
        }
    }

    fn preflight_row<T, K, Message>(
        template: &KeyedListTemplate<T, K, Message>,
        element: Element,
        item: &T,
    ) -> Result<PreparedRow, JsValue> {
        if !element.tag_name().eq_ignore_ascii_case("li") {
            return Err(JsValue::from_str("M7b SSR row tag mismatch"));
        }
        for (name, value) in &template.row_attributes {
            if element.get_attribute(name).as_deref() != Some(value.as_str()) {
                return Err(JsValue::from_str("M7b SSR row attribute mismatch"));
            }
        }
        for attribute in &template.dynamic_attributes {
            if element.get_attribute(attribute.name) != (attribute.value)(item) {
                return Err(JsValue::from_str("M7b SSR dynamic attribute mismatch"));
            }
        }

        let conditional_value = template
            .conditional
            .as_ref()
            .and_then(|spec| (spec.value)(item));
        let expected_children = 1
            + usize::from(template.input.is_some())
            + usize::from(conditional_value.is_some())
            + usize::from(template.action.is_some());
        let children = direct_element_children(&element, "row")?;
        if children.len() != expected_children {
            return Err(JsValue::from_str("M7b SSR row shape mismatch"));
        }

        let label = children
            .first()
            .cloned()
            .ok_or_else(|| JsValue::from_str("M7b SSR label slot missing"))?;
        let label_value = (template.label)(item);
        if !label.tag_name().eq_ignore_ascii_case("span")
            || label.get_attribute("data-yetlet-slot").as_deref() != Some("label")
            || !exact_text_node(&label, &label_value)
        {
            return Err(JsValue::from_str("M7b SSR label slot mismatch"));
        }

        let mut index = 1;
        let (input, input_model_value) = if let Some(spec) = &template.input {
            let node = children
                .get(index)
                .cloned()
                .ok_or_else(|| JsValue::from_str("M7d SSR input slot missing"))?;
            index += 1;
            let expected = (spec.value)(item);
            if !node.tag_name().eq_ignore_ascii_case("input")
                || node.get_attribute("type").as_deref() != Some("text")
                || node.get_attribute("data-yetlet-input").as_deref() != Some(spec.slot)
                || node.get_attribute("aria-label").as_deref() != Some(spec.aria_label.as_str())
                || node.get_attribute("value").as_deref() != Some(expected.as_str())
                || node.first_child().is_some()
            {
                return Err(JsValue::from_str("M7d SSR input slot mismatch"));
            }
            let input: HtmlInputElement = node
                .dyn_into()
                .map_err(|_| JsValue::from_str("M7d SSR input conversion failed"))?;
            if input.value() != expected {
                return Err(JsValue::from_str("M7d SSR live input value mismatch"));
            }
            (Some(input), Some(expected))
        } else {
            (None, None)
        };

        let conditional = match (&template.conditional, conditional_value) {
            (Some(spec), Some(value)) => {
                let node = children
                    .get(index)
                    .cloned()
                    .ok_or_else(|| JsValue::from_str("M7b SSR conditional slot missing"))?;
                index += 1;
                if !node.tag_name().eq_ignore_ascii_case("span")
                    || node.get_attribute("data-yetlet-slot").as_deref() != Some(spec.slot)
                    || !exact_text_node(&node, &value)
                {
                    return Err(JsValue::from_str("M7b SSR conditional slot mismatch"));
                }
                match spec.class {
                    Some(class_name)
                        if node.get_attribute("class").as_deref() == Some(class_name) => {}
                    None if !node.has_attribute("class") => {}
                    _ => {
                        return Err(JsValue::from_str("M7b SSR conditional class mismatch"));
                    }
                }
                Some(node)
            }
            (Some(_), None) | (None, None) => None,
            (None, Some(_)) => unreachable!("conditional value requires a conditional spec"),
        };

        let action = if let Some(spec) = &template.action {
            let node = children
                .get(index)
                .cloned()
                .ok_or_else(|| JsValue::from_str("M7b SSR row action missing"))?;
            if !node.tag_name().eq_ignore_ascii_case("button")
                || node.get_attribute("type").as_deref() != Some("button")
                || node.get_attribute("data-yetlet-action").as_deref() != Some("row")
                || !exact_text_node(&node, &spec.label)
            {
                return Err(JsValue::from_str("M7b SSR row action mismatch"));
            }
            Some(node)
        } else {
            None
        };

        Ok(PreparedRow {
            element,
            label,
            input,
            input_model_value,
            conditional,
            action,
        })
    }

    fn bind_prepared_row<T, K, Message>(
        template: &KeyedListTemplate<T, K, Message>,
        prepared: PreparedRow,
        key: K,
        dispatch: Rc<RefCell<Box<dyn FnMut(Message)>>>,
    ) -> Result<KeyedEntry<BoundRow>, JsValue>
    where
        K: Clone + ToString + 'static,
        Message: 'static,
    {
        let PreparedRow {
            element,
            label,
            input,
            input_model_value,
            conditional,
            action,
        } = prepared;
        let mut bindings = Vec::with_capacity(2);
        if let Some(binding) = attach_input(
            template.input.as_ref(),
            input.as_ref(),
            key.clone(),
            Rc::clone(&dispatch),
        )? {
            bindings.push(binding);
        }
        if let Some(binding) =
            attach_action(template.action.as_ref(), action.as_ref(), key, dispatch)?
        {
            bindings.push(binding);
        }
        Ok(KeyedEntry::new(
            element,
            BoundRow {
                label,
                input,
                input_model_value,
                conditional,
                action,
                _bindings: bindings,
            },
        ))
    }

    fn create_row<T, K, Message>(
        template: &KeyedListTemplate<T, K, Message>,
        document: &Document,
        stable_key: &BindingKey,
        item: &T,
        event_key: K,
        dispatch: Rc<RefCell<Box<dyn FnMut(Message)>>>,
    ) -> Result<KeyedEntry<BoundRow>, JsValue>
    where
        K: Clone + ToString + 'static,
        Message: 'static,
    {
        let element = document.create_element("li")?;
        element.set_attribute("data-yetlet-key", stable_key.as_str())?;
        for (name, value) in &template.row_attributes {
            element.set_attribute(name, value)?;
        }
        let label = document.create_element("span")?;
        label.set_attribute("data-yetlet-slot", "label")?;
        element.append_child(&label)?;
        let (input, input_model_value) = if let Some(spec) = &template.input {
            let expected = (spec.value)(item);
            let node: HtmlInputElement = document.create_element("input")?.dyn_into()?;
            node.set_attribute("type", "text")?;
            node.set_attribute("data-yetlet-input", spec.slot)?;
            node.set_attribute("aria-label", &spec.aria_label)?;
            node.set_attribute("value", &expected)?;
            node.set_value(&expected);
            element.append_child(&node)?;
            (Some(node), Some(expected))
        } else {
            (None, None)
        };
        let action = if let Some(spec) = &template.action {
            let button = document.create_element("button")?;
            button.set_attribute("type", "button")?;
            button.set_attribute("data-yetlet-action", "row")?;
            button.set_text_content(Some(&spec.label));
            element.append_child(&button)?;
            Some(button)
        } else {
            None
        };
        let mut bindings = Vec::with_capacity(2);
        if let Some(binding) = attach_input(
            template.input.as_ref(),
            input.as_ref(),
            event_key.clone(),
            Rc::clone(&dispatch),
        )? {
            bindings.push(binding);
        }
        if let Some(binding) = attach_action(
            template.action.as_ref(),
            action.as_ref(),
            event_key,
            dispatch,
        )? {
            bindings.push(binding);
        }
        let mut row = BoundRow {
            label,
            input,
            input_model_value,
            conditional: None,
            action,
            _bindings: bindings,
        };
        update_row(template, stable_key, &element, &mut row, item)?;
        Ok(KeyedEntry::new(element, row))
    }

    fn attach_action<K, Message>(
        action: Option<&RowAction<K, Message>>,
        target: Option<&Element>,
        key: K,
        dispatch: Rc<RefCell<Box<dyn FnMut(Message)>>>,
    ) -> Result<Option<DomEventListener>, JsValue>
    where
        K: Clone + 'static,
        Message: 'static,
    {
        let (Some(action), Some(target)) = (action, target) else {
            return Ok(None);
        };
        let message = Rc::clone(&action.message);
        Ok(Some(DomEventListener::new(
            target.as_ref(),
            "click",
            move |_event| {
                let value = message(key.clone());
                (dispatch.borrow_mut())(value);
            },
        )?))
    }

    fn attach_input<T, K, Message>(
        input: Option<&RowInput<T, K, Message>>,
        target: Option<&HtmlInputElement>,
        key: K,
        dispatch: Rc<RefCell<Box<dyn FnMut(Message)>>>,
    ) -> Result<Option<DomEventListener>, JsValue>
    where
        K: Clone + 'static,
        Message: 'static,
    {
        let (Some(input), Some(target)) = (input, target) else {
            return Ok(None);
        };
        let message = Rc::clone(&input.message);
        let live_input = target.clone();
        Ok(Some(DomEventListener::new(
            target.as_ref(),
            "input",
            move |_event| {
                let value = message(key.clone(), live_input.value());
                (dispatch.borrow_mut())(value);
            },
        )?))
    }

    fn update_row<T, K, Message>(
        template: &KeyedListTemplate<T, K, Message>,
        stable_key: &BindingKey,
        element: &Element,
        row: &mut BoundRow,
        item: &T,
    ) -> Result<(), JsValue>
    where
        K: ToString,
    {
        let expected_key = binding_key((template.key)(item).to_string())?;
        if &expected_key != stable_key
            || element.get_attribute("data-yetlet-key").as_deref() != Some(stable_key.as_str())
        {
            return Err(JsValue::from_str("M7b keyed row identity changed"));
        }
        for attribute in &template.dynamic_attributes {
            match (attribute.value)(item) {
                Some(value)
                    if element.get_attribute(attribute.name).as_deref() != Some(value.as_str()) =>
                {
                    element.set_attribute(attribute.name, &value)?;
                }
                None if element.has_attribute(attribute.name) => {
                    element.remove_attribute(attribute.name)?;
                }
                _ => {}
            }
        }
        let label = (template.label)(item);
        if row.label.text_content().as_deref() != Some(&label) {
            row.label.set_text_content(Some(&label));
        }
        match (
            template.input.as_ref(),
            row.input.as_ref(),
            row.input_model_value.as_mut(),
        ) {
            (Some(spec), Some(input), Some(previous)) => {
                let expected = (spec.value)(item);
                if *previous != expected {
                    if input.value() != expected {
                        input.set_value(&expected);
                    }
                    if input.get_attribute("value").as_deref() != Some(expected.as_str()) {
                        input.set_attribute("value", &expected)?;
                    }
                    *previous = expected;
                }
            }
            (None, None, None) => {}
            _ => return Err(JsValue::from_str("M7d input binding shape mismatch")),
        }

        if let Some(spec) = &template.conditional {
            match (spec.value)(item) {
                Some(value) => {
                    if let Some(node) = &row.conditional {
                        if node.text_content().as_deref() != Some(&value) {
                            node.set_text_content(Some(&value));
                        }
                    } else {
                        let document = element
                            .owner_document()
                            .ok_or_else(|| JsValue::from_str("M7b row has no document"))?;
                        let node = document.create_element("span")?;
                        node.set_attribute("data-yetlet-slot", spec.slot)?;
                        if let Some(class_name) = spec.class {
                            node.set_attribute("class", class_name)?;
                        }
                        node.set_text_content(Some(&value));
                        let reference: Option<Node> = row.action.clone().map(Into::into);
                        element.insert_before(&node, reference.as_ref())?;
                        row.conditional = Some(node);
                    }
                }
                None => {
                    if let Some(node) = row.conditional.take() {
                        element.remove_child(&node)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn binding_key(value: impl Into<String>) -> Result<BindingKey, JsValue> {
        BindingKey::new(value).map_err(|error| JsValue::from_str(&error.to_string()))
    }

    fn template_error(error: TemplateError) -> JsValue {
        JsValue::from_str(&error.to_string())
    }

    type DemoGraph = LocalGraph<Vec<DemoItem>, DemoMessage, String>;
    type DemoGraphHandle = GraphHandle<Vec<DemoItem>, DemoMessage, String>;
    type DemoSubscription = Subscription<Vec<DemoItem>, DemoMessage, String>;
    type DemoBinding = BoundKeyedList<DemoItem, u64, DemoMessage>;

    struct DemoRuntime {
        graph: DemoGraph,
        binding: Rc<RefCell<Option<DemoBinding>>>,
        _subscription: DemoSubscription,
    }

    thread_local! {
        static DEMO: RefCell<Option<DemoRuntime>> = const { RefCell::new(None) };
    }

    fn document() -> Result<Document, JsValue> {
        web_sys::window()
            .ok_or_else(|| JsValue::from_str("window unavailable"))?
            .document()
            .ok_or_else(|| JsValue::from_str("document unavailable"))
    }

    fn reactive_error(error: ReactiveError<String>) -> JsValue {
        JsValue::from_str(&error.to_string())
    }

    fn js_error(error: JsValue) -> String {
        error
            .as_string()
            .unwrap_or_else(|| "browser DOM operation failed".to_owned())
    }

    fn record_stats(name: &str, stats: FlushStats) -> Result<(), JsValue> {
        let root = document()?
            .document_element()
            .ok_or_else(|| JsValue::from_str("document element missing"))?;
        root.set_attribute(
            &format!("data-yetlet-m7c-{name}-messages"),
            &stats.messages.to_string(),
        )?;
        root.set_attribute(
            &format!("data-yetlet-m7c-{name}-passes"),
            &stats.passes.to_string(),
        )?;
        root.set_attribute(
            &format!("data-yetlet-m7c-{name}-commits"),
            &stats.commits.to_string(),
        )?;
        Ok(())
    }

    fn with_runtime<Output>(
        operation: impl FnOnce(&DemoRuntime) -> Result<Output, JsValue>,
    ) -> Result<Output, JsValue> {
        DEMO.with(|runtime| {
            let runtime = runtime.borrow();
            operation(
                runtime
                    .as_ref()
                    .ok_or_else(|| JsValue::from_str("M7c demo is not mounted"))?,
            )
        })
    }

    fn dispatch_demo(message: DemoMessage) -> Result<FlushStats, JsValue> {
        with_runtime(|runtime| {
            runtime
                .graph
                .handle()
                .dispatch(message)
                .map_err(reactive_error)
        })
    }

    #[wasm_bindgen]
    pub fn verify_disposed_handle_isolation() -> Result<(), JsValue> {
        dispose_demo()?;
        let document = document()?;
        let host = document
            .get_element_by_id("prototype-host")
            .ok_or_else(|| JsValue::from_str("M7b prototype host missing"))?;
        let items = demo_items();
        let mut retired =
            BoundKeyedList::bind_existing(demo_template(), &host, &items, |_message| {})?;
        retired.dispose()?;
        let mut current =
            BoundKeyedList::bind_existing(demo_template(), &host, &items, |_message| {})?;
        drop(retired);
        if current.root.get_attribute("data-yetlet-bound").as_deref() != Some("true") {
            current.dispose()?;
            return Err(JsValue::from_str(
                "M7b disposed handle removed a newer binding marker",
            ));
        }
        match BoundKeyedList::bind_existing(demo_template(), &host, &items, |_message| {}) {
            Err(error)
                if error
                    .as_string()
                    .is_some_and(|message| message.contains("already bound")) =>
            {
                current.dispose()?;
                Ok(())
            }
            Err(error) => {
                current.dispose()?;
                Err(error)
            }
            Ok(mut duplicate) => {
                duplicate.dispose()?;
                current.dispose()?;
                Err(JsValue::from_str(
                    "M7b accepted a concurrent binding after retired handle drop",
                ))
            }
        }
    }

    #[wasm_bindgen]
    pub fn mount_demo() -> Result<(), JsValue> {
        dispose_demo()?;
        let document = document()?;
        let host = document
            .get_element_by_id("prototype-host")
            .ok_or_else(|| JsValue::from_str("M7c prototype host missing"))?;
        let root = document
            .document_element()
            .ok_or_else(|| JsValue::from_str("document element missing"))?;
        root.remove_attribute("data-yetlet-m7d-event-error")?;
        let binding: Rc<RefCell<Option<DemoBinding>>> = Rc::new(RefCell::new(None));
        let weak_binding = Rc::downgrade(&binding);
        let commit_count = Rc::new(Cell::new(0_u32));
        let commit_counter = Rc::clone(&commit_count);
        let commit_root = root.clone();
        let graph = LocalGraph::new(demo_items(), reduce_demo, move |items| {
            let binding = weak_binding
                .upgrade()
                .ok_or_else(|| "M7c binding owner is gone".to_owned())?;
            binding
                .borrow_mut()
                .as_mut()
                .ok_or_else(|| "M7c keyed binding is missing".to_owned())?
                .reconcile(items)
                .map_err(js_error)?;
            let count = commit_counter.get() + 1;
            commit_counter.set(count);
            commit_root
                .set_attribute("data-yetlet-m7c-commit-count", &count.to_string())
                .map_err(js_error)?;
            Ok(())
        });
        let dispatch_handle = graph.handle();
        let event_dispatch = dispatch_handle.clone();
        let event_root = root.clone();
        let initial = graph.snapshot().map_err(reactive_error)?;
        let bound =
            BoundKeyedList::bind_existing(demo_template(), &host, &initial, move |message| {
                if let Err(error) = event_dispatch.dispatch(message) {
                    let message = error.to_string();
                    let _ = event_root.set_attribute("data-yetlet-m7d-event-error", &message);
                    web_sys::console::error_1(&JsValue::from_str(&message));
                }
            })?;
        *binding.borrow_mut() = Some(bound);

        let notification_count = Rc::new(Cell::new(0_u32));
        let observer_count = Rc::clone(&notification_count);
        let observer_root = root.clone();
        let subscription = graph
            .subscribe(move |_items| {
                let count = observer_count.get() + 1;
                observer_count.set(count);
                let _ = observer_root
                    .set_attribute("data-yetlet-m7c-notification-count", &count.to_string());
            })
            .map_err(reactive_error)?;

        root.set_attribute("data-yetlet-m7b-bound", "true")?;
        root.set_attribute("data-yetlet-m7c-owned", "true")?;
        root.set_attribute("data-yetlet-m7c-commit-count", "0")?;
        root.set_attribute("data-yetlet-m7c-notification-count", "0")?;
        DEMO.with(|runtime| {
            *runtime.borrow_mut() = Some(DemoRuntime {
                graph,
                binding,
                _subscription: subscription,
            });
        });
        Ok(())
    }

    #[wasm_bindgen]
    pub fn reverse_demo() -> Result<(), JsValue> {
        dispatch_demo(DemoMessage::Reverse).map(|_| ())
    }

    #[wasm_bindgen]
    pub fn append_demo() -> Result<(), JsValue> {
        dispatch_demo(DemoMessage::Append).map(|_| ())
    }

    #[wasm_bindgen]
    pub fn remove_first_demo() -> Result<(), JsValue> {
        dispatch_demo(DemoMessage::RemoveFirst).map(|_| ())
    }

    #[wasm_bindgen]
    pub fn reset_demo() -> Result<(), JsValue> {
        dispatch_demo(DemoMessage::Reset).map(|_| ())
    }

    #[wasm_bindgen]
    pub fn batch_demo() -> Result<(), JsValue> {
        let stats = with_runtime(|runtime| {
            runtime
                .graph
                .handle()
                .batch([
                    DemoMessage::Toggle(1),
                    DemoMessage::Toggle(2),
                    DemoMessage::Reverse,
                ])
                .map_err(reactive_error)
        })?;
        record_stats("batch", stats)
    }

    #[wasm_bindgen]
    pub fn reentrant_demo() -> Result<(), JsValue> {
        let stats = with_runtime(|runtime| {
            let handle = runtime.graph.handle();
            let reentrant = handle.clone();
            let queued = Rc::new(Cell::new(false));
            let queued_once = Rc::clone(&queued);
            let error = Rc::new(RefCell::new(None::<String>));
            let observer_error = Rc::clone(&error);
            let subscription = runtime
                .graph
                .subscribe(move |_items| {
                    if !queued_once.replace(true) {
                        if let Err(value) = reentrant.dispatch(DemoMessage::Toggle(3)) {
                            *observer_error.borrow_mut() = Some(value.to_string());
                        }
                    }
                })
                .map_err(reactive_error)?;
            let stats = handle
                .dispatch(DemoMessage::Toggle(1))
                .map_err(reactive_error)?;
            drop(subscription);
            if let Some(error) = error.borrow_mut().take() {
                return Err(JsValue::from_str(&error));
            }
            Ok(stats)
        })?;
        record_stats("reentrant", stats)
    }

    #[wasm_bindgen]
    pub fn verify_subscription_disposal() -> Result<(), JsValue> {
        with_runtime(|runtime| {
            let calls = Rc::new(Cell::new(0_u32));
            let observed = Rc::clone(&calls);
            let mut subscription = runtime
                .graph
                .subscribe(move |_items| observed.set(observed.get() + 1))
                .map_err(reactive_error)?;
            if !subscription.unsubscribe() {
                return Err(JsValue::from_str("M7c subscription did not unsubscribe"));
            }
            let stats = runtime
                .graph
                .handle()
                .batch([DemoMessage::Toggle(1), DemoMessage::Toggle(1)])
                .map_err(reactive_error)?;
            if calls.get() != 0 || stats.commits != 1 {
                return Err(JsValue::from_str(
                    "M7c disposed subscription was notified or batch was not coalesced",
                ));
            }
            Ok(())
        })
    }

    #[wasm_bindgen]
    pub fn verify_reducer_failure() -> Result<(), JsValue> {
        with_runtime(|runtime| {
            let before = runtime.graph.snapshot().map_err(reactive_error)?;
            let root = document()?
                .document_element()
                .ok_or_else(|| JsValue::from_str("document element missing"))?;
            let commits = root
                .get_attribute("data-yetlet-m7c-commit-count")
                .unwrap_or_default();
            match runtime.graph.handle().dispatch(DemoMessage::Reject) {
                Err(ReactiveError::Reducer(_)) => {}
                Err(error) => return Err(reactive_error(error)),
                Ok(_) => return Err(JsValue::from_str("M7c reducer failure was accepted")),
            }
            if runtime.graph.snapshot().map_err(reactive_error)? != before
                || root
                    .get_attribute("data-yetlet-m7c-commit-count")
                    .unwrap_or_default()
                    != commits
            {
                return Err(JsValue::from_str(
                    "M7c reducer failure changed committed state or DOM",
                ));
            }
            runtime
                .graph
                .handle()
                .batch([DemoMessage::Toggle(1), DemoMessage::Toggle(1)])
                .map_err(reactive_error)?;
            Ok(())
        })
    }

    #[wasm_bindgen]
    pub fn verify_commit_failure_isolation() -> Result<(), JsValue> {
        let document = document()?;
        let (items, source_root) = with_runtime(|runtime| {
            let items = runtime.graph.snapshot().map_err(reactive_error)?;
            let binding = runtime.binding.borrow();
            let root = binding
                .as_ref()
                .ok_or_else(|| JsValue::from_str("M7c keyed binding is missing"))?
                .root
                .clone();
            Ok((items, root))
        })?;
        let detached = document.create_element("div")?;
        let cloned: Element = source_root.clone_node_with_deep(true)?.dyn_into()?;
        cloned.remove_attribute("data-yetlet-bound")?;
        detached.append_child(&cloned)?;
        let row_before = cloned
            .query_selector("[data-yetlet-key=\"2\"]")?
            .ok_or_else(|| JsValue::from_str("M7c detached keyed row missing"))?;
        let binding: Rc<RefCell<Option<DemoBinding>>> = Rc::new(RefCell::new(None));
        let weak_binding = Rc::downgrade(&binding);
        let mut graph = LocalGraph::new(items.clone(), reduce_demo, move |next| {
            let binding = weak_binding
                .upgrade()
                .ok_or_else(|| "M7c detached binding owner is gone".to_owned())?;
            binding
                .borrow_mut()
                .as_mut()
                .ok_or_else(|| "M7c detached binding is missing".to_owned())?
                .reconcile(next)
                .map_err(js_error)
        });
        let bound =
            BoundKeyedList::bind_existing(demo_template(), &detached, &items, |_message| {})?;
        *binding.borrow_mut() = Some(bound);
        let handle = graph.handle();
        match handle.dispatch(DemoMessage::InjectDuplicate) {
            Err(ReactiveError::Commit(_)) => {}
            Err(error) => return Err(reactive_error(error)),
            Ok(_) => return Err(JsValue::from_str("M7c invalid commit was accepted")),
        }
        let row_after = cloned
            .query_selector("[data-yetlet-key=\"2\"]")?
            .ok_or_else(|| JsValue::from_str("M7c detached keyed row disappeared"))?;
        if !row_before.is_same_node(Some(&row_after))
            || row_after.get_attribute("data-yetlet-key").as_deref() != Some("2")
        {
            return Err(JsValue::from_str(
                "M7c failed commit corrupted keyed DOM identity",
            ));
        }
        if !matches!(
            handle.dispatch(DemoMessage::Toggle(1)),
            Err(ReactiveError::Faulted)
        ) {
            return Err(JsValue::from_str("M7c failed graph did not remain faulted"));
        }
        graph.dispose();
        if let Some(mut binding) = binding.borrow_mut().take() {
            binding.dispose()?;
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn verify_graph_disposal() -> Result<(), JsValue> {
        let handle: DemoGraphHandle = with_runtime(|runtime| Ok(runtime.graph.handle()))?;
        dispose_demo()?;
        match handle.dispatch(DemoMessage::Toggle(1)) {
            Err(ReactiveError::Disposed) => Ok(()),
            Err(error) => Err(reactive_error(error)),
            Ok(_) => Err(JsValue::from_str(
                "M7c weak dispatch survived owner disposal",
            )),
        }
    }

    #[wasm_bindgen]
    pub fn dispose_demo() -> Result<(), JsValue> {
        DEMO.with(|runtime| {
            if let Some(mut runtime) = runtime.borrow_mut().take() {
                runtime.graph.dispose();
                if let Some(mut binding) = runtime.binding.borrow_mut().take() {
                    binding.dispose()?;
                }
            }
            Ok::<(), JsValue>(())
        })?;
        if let Ok(document) = document() {
            if let Some(root) = document.document_element() {
                for attribute in [
                    "data-yetlet-m7b-bound",
                    "data-yetlet-m7c-owned",
                    "data-yetlet-m7c-commit-count",
                    "data-yetlet-m7c-notification-count",
                    "data-yetlet-m7c-batch-messages",
                    "data-yetlet-m7c-batch-passes",
                    "data-yetlet-m7c-batch-commits",
                    "data-yetlet-m7c-reentrant-messages",
                    "data-yetlet-m7c-reentrant-passes",
                    "data-yetlet-m7c-reentrant-commits",
                    "data-yetlet-m7d-event-error",
                ] {
                    root.remove_attribute(attribute)?;
                }
            }
        }
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use browser::{
    BoundKeyedList, append_demo, batch_demo, dispose_demo, mount_demo, reentrant_demo,
    remove_first_demo, reset_demo, reverse_demo, verify_commit_failure_isolation,
    verify_disposed_handle_isolation, verify_graph_disposal, verify_reducer_failure,
    verify_subscription_disposal,
};

#[cfg(test)]
mod tests {
    use super::{
        BindingContractError, ComponentIdentity, DemoItem, KeyedListTemplate, TemplateError,
        demo_items, demo_ssr_html, demo_template,
    };

    #[test]
    fn identity_and_attribute_contracts_fail_closed() {
        assert!(ComponentIdentity::new("todo-list", "instance-1").is_ok());
        assert_eq!(
            ComponentIdentity::new("TodoList", "instance-1"),
            Err(BindingContractError::InvalidComponentName)
        );
        let template = KeyedListTemplate::<DemoItem, u64, ()>::new(
            ComponentIdentity::new("todo-list", "demo").unwrap(),
            |item| item.id,
            |item| item.title.clone(),
        );
        assert!(matches!(
            template.dynamic_attribute("onclick", |_| Some("bad".into())),
            Err(TemplateError::UnsupportedAttribute(_))
        ));
        let template = KeyedListTemplate::<DemoItem, u64, ()>::new(
            ComponentIdentity::new("todo-list", "demo").unwrap(),
            |item| item.id,
            |item| item.title.clone(),
        );
        assert!(matches!(
            template.list_attribute("data-yetlet-instance", "override"),
            Err(TemplateError::UnsupportedAttribute(_))
        ));
        let template = KeyedListTemplate::<DemoItem, u64, ()>::new(
            ComponentIdentity::new("todo-list", "demo").unwrap(),
            |item| item.id,
            |item| item.title.clone(),
        );
        assert!(matches!(
            template.dynamic_attribute("data-YETLET-key", |_| Some("override".into())),
            Err(TemplateError::UnsupportedAttribute(_))
        ));
    }

    #[test]
    fn authoring_contract_rejects_ambiguous_slots_attributes_actions_and_keys() {
        let identity = || ComponentIdentity::new("todo-list", "demo").unwrap();
        let template = KeyedListTemplate::<DemoItem, u64, ()>::new(
            identity(),
            |item| item.id,
            |item| item.title.clone(),
        )
        .row_attribute("class", "row")
        .unwrap();
        assert!(matches!(
            template.dynamic_attribute("class", |_| Some("dynamic".into())),
            Err(TemplateError::DuplicateAttribute(_))
        ));

        let template = KeyedListTemplate::<DemoItem, u64, ()>::new(
            identity(),
            |item| item.id,
            |item| item.title.clone(),
        );
        assert!(matches!(
            template.conditional_text("label", None, |_| Some("ambiguous".into())),
            Err(TemplateError::InvalidSlot("name"))
        ));

        let template = KeyedListTemplate::<DemoItem, u64, ()>::new(
            identity(),
            |item| item.id,
            |item| item.title.clone(),
        );
        assert!(matches!(
            template.on_row_click(" \n", |_| ()),
            Err(TemplateError::InvalidAction("label"))
        ));

        let template = KeyedListTemplate::<DemoItem, u64, ()>::new(
            identity(),
            |item| item.id,
            |item| item.title.clone(),
        );
        assert!(matches!(
            template.text_input("draft", " \n", |item| item.draft.clone(), |_, _| ()),
            Err(TemplateError::InvalidInput("aria-label"))
        ));

        let template = KeyedListTemplate::<DemoItem, u64, ()>::new(
            identity(),
            |item| item.id,
            |item| item.title.clone(),
        )
        .text_input("draft", "Draft", |item| item.draft.clone(), |_, _| ())
        .unwrap();
        assert!(matches!(
            template.text_input("other", "Other", |item| item.draft.clone(), |_, _| ()),
            Err(TemplateError::InvalidInput("duplicate"))
        ));

        let template = KeyedListTemplate::<DemoItem, u64, ()>::new(
            identity(),
            |item| item.id,
            |item| item.title.clone(),
        )
        .text_input("draft", "Draft", |item| item.draft.clone(), |_, _| ())
        .unwrap();
        assert!(matches!(
            template.conditional_text("draft", None, |_| Some("ambiguous".into())),
            Err(TemplateError::InvalidSlot("name"))
        ));

        let template = KeyedListTemplate::<DemoItem, String, ()>::new(
            identity(),
            |_| "bad\nkey".to_owned(),
            |item| item.title.clone(),
        );
        assert!(matches!(
            template.ssr_view(&demo_items()),
            Err(TemplateError::InvalidKey(_))
        ));
    }

    #[test]
    fn ssr_contains_stable_identity_slots_conditionals_and_no_inline_event() {
        let html = demo_ssr_html().unwrap();
        assert!(html.contains("data-yetlet-component=\"todo-list\""));
        assert!(html.contains("data-yetlet-instance=\"m7b-demo\""));
        assert_eq!(html.matches("data-yetlet-key=").count(), 3);
        assert_eq!(html.matches("data-yetlet-slot=\"label\"").count(), 3);
        assert_eq!(html.matches("data-yetlet-input=\"draft\"").count(), 3);
        assert_eq!(html.matches("data-yetlet-slot=\"details\"").count(), 1);
        assert_eq!(html.matches("data-yetlet-action=\"row\"").count(), 3);
        assert!(!html.contains("onclick="));
        assert!(!html.contains("oninput="));
    }

    #[test]
    fn ssr_escapes_dynamic_text_and_rejects_duplicate_keys() {
        let template = demo_template();
        let mut items = demo_items();
        items[0].title = "<img src=x onerror=1> &".into();
        items[0].draft = "<script>alert(\"x\")</script> &".into();
        let html = yetlet_html::render_to_string(&template.ssr_view(&items).unwrap()).unwrap();
        assert!(html.contains("&lt;img src=x onerror=1&gt; &amp;"));
        assert!(html.contains("value=\"&lt;script&gt;alert(&quot;x&quot;)&lt;/script&gt; &amp;\""));
        items[1].id = items[0].id;
        assert!(matches!(
            template.ssr_view(&items),
            Err(TemplateError::DuplicateKey(_))
        ));
    }
}
