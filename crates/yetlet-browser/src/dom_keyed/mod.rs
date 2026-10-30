//! General keyed DOM ownership and reconciliation primitives.
//!
//! The primitive preserves caller-owned element state by semantic key. It is
//! deliberately lower-level than a component/template API: callers provide key,
//! create, and update functions, while this module owns ordering, node identity,
//! listener lifetimes, and explicit disposal.

use std::collections::HashSet;

use yetlet::view::binding::BindingKey;

/// Failure returned before a keyed order is accepted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyedOrderError {
    DuplicateKey(String),
}

impl std::fmt::Display for KeyedOrderError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateKey(key) => write!(formatter, "duplicate keyed DOM item: {key}"),
        }
    }
}

impl std::error::Error for KeyedOrderError {}

/// Validate a complete desired order before any DOM mutation.
///
/// The original order is retained. Individual key syntax is validated by
/// [`BindingKey`]; this function adds set-level uniqueness.
pub fn validate_keyed_order(
    keys: impl IntoIterator<Item = BindingKey>,
) -> Result<Vec<BindingKey>, KeyedOrderError> {
    let keys = keys.into_iter().collect::<Vec<_>>();
    let mut seen = HashSet::with_capacity(keys.len());
    for key in &keys {
        if !seen.insert(key.as_str().to_owned()) {
            return Err(KeyedOrderError::DuplicateKey(key.as_str().to_owned()));
        }
    }
    Ok(keys)
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use std::collections::{HashMap, HashSet};

    use wasm_bindgen::{JsCast, JsValue, closure::Closure};
    use web_sys::{Document, Element, Event, EventTarget, Node};

    use super::{BindingKey, validate_keyed_order};

    fn keyed_error(error: impl std::fmt::Display) -> JsValue {
        JsValue::from_str(&error.to_string())
    }

    fn valid_event_type(value: &str) -> bool {
        !value.is_empty()
            && value
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte == b'-')
    }

    /// Owns one browser event listener and removes it on explicit disposal or drop.
    pub struct DomEventListener {
        target: EventTarget,
        event_type: String,
        callback: Option<Closure<dyn FnMut(Event)>>,
    }

    impl DomEventListener {
        pub fn new(
            target: &EventTarget,
            event_type: impl Into<String>,
            handler: impl FnMut(Event) + 'static,
        ) -> Result<Self, JsValue> {
            let event_type = event_type.into();
            if !valid_event_type(&event_type) {
                return Err(JsValue::from_str("invalid DOM event type"));
            }
            let callback = Closure::<dyn FnMut(Event)>::new(handler);
            target
                .add_event_listener_with_callback(&event_type, callback.as_ref().unchecked_ref())?;
            Ok(Self {
                target: target.clone(),
                event_type,
                callback: Some(callback),
            })
        }

        pub fn is_active(&self) -> bool {
            self.callback.is_some()
        }

        pub fn dispose(&mut self) -> Result<bool, JsValue> {
            let Some(callback) = self.callback.as_ref() else {
                return Ok(false);
            };
            self.target.remove_event_listener_with_callback(
                &self.event_type,
                callback.as_ref().unchecked_ref(),
            )?;
            self.callback.take();
            Ok(true)
        }
    }

    impl Drop for DomEventListener {
        fn drop(&mut self) {
            if let Some(callback) = self.callback.as_ref() {
                let _ = self.target.remove_event_listener_with_callback(
                    &self.event_type,
                    callback.as_ref().unchecked_ref(),
                );
            }
        }
    }

    /// One keyed element plus caller-defined state such as cached slots/listeners.
    pub struct KeyedEntry<State> {
        element: Element,
        state: State,
    }

    impl<State> KeyedEntry<State> {
        pub fn new(element: Element, state: State) -> Self {
            Self { element, state }
        }

        pub fn element(&self) -> &Element {
            &self.element
        }

        pub fn state(&self) -> &State {
            &self.state
        }

        pub fn state_mut(&mut self) -> &mut State {
            &mut self.state
        }

        fn parts_mut(&mut self) -> (&Element, &mut State) {
            (&self.element, &mut self.state)
        }
    }

    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
    pub struct ReconcileStats {
        pub retained: usize,
        pub inserted: usize,
        pub removed: usize,
        pub moved: usize,
    }

    /// Owns keyed element state under one root and preserves identity on reorder.
    ///
    /// Key validation and creation of missing detached entries complete before
    /// structural mutation. Caller updates run before structural changes but may
    /// themselves mutate retained nodes; an update error therefore has the
    /// caller's atomicity semantics. Unexpected DOM API failure during the final
    /// remove/insert phase is not transactionally rolled back.
    pub struct KeyedChildren<State> {
        root: Element,
        entries: HashMap<BindingKey, KeyedEntry<State>>,
        disposed: bool,
    }

    impl<State> KeyedChildren<State> {
        pub fn from_existing(
            root: Element,
            entries: Vec<(BindingKey, KeyedEntry<State>)>,
        ) -> Result<Self, JsValue> {
            let order = validate_keyed_order(entries.iter().map(|(key, _)| key.clone()))
                .map_err(keyed_error)?;
            let children = direct_element_children(&root)?;
            if children.len() != entries.len() {
                return Err(JsValue::from_str(
                    "keyed DOM entry set does not match direct children",
                ));
            }
            for (((expected_key, entry), validated_key), child) in
                entries.iter().zip(order.iter()).zip(children.iter())
            {
                if expected_key != validated_key || !entry.element.is_same_node(Some(child)) {
                    return Err(JsValue::from_str(
                        "keyed DOM entry order does not match direct children",
                    ));
                }
            }
            Ok(Self {
                root,
                entries: entries.into_iter().collect(),
                disposed: false,
            })
        }

        pub fn empty(root: Element) -> Result<Self, JsValue> {
            if root.first_child().is_some() {
                return Err(JsValue::from_str("empty keyed DOM root contains children"));
            }
            Ok(Self {
                root,
                entries: HashMap::new(),
                disposed: false,
            })
        }

        pub fn root(&self) -> &Element {
            &self.root
        }

        pub fn len(&self) -> usize {
            self.entries.len()
        }

        pub fn is_empty(&self) -> bool {
            self.entries.is_empty()
        }

        pub fn is_disposed(&self) -> bool {
            self.disposed
        }

        pub fn reconcile<T>(
            &mut self,
            items: &[T],
            mut key_for: impl FnMut(&T) -> Result<BindingKey, JsValue>,
            mut update: impl FnMut(&BindingKey, &Element, &mut State, &T) -> Result<(), JsValue>,
            mut create: impl FnMut(&Document, &BindingKey, &T) -> Result<KeyedEntry<State>, JsValue>,
        ) -> Result<ReconcileStats, JsValue> {
            if self.disposed {
                return Err(JsValue::from_str("keyed DOM owner is disposed"));
            }

            let desired = items
                .iter()
                .map(|item| key_for(item).map(|key| (key, item)))
                .collect::<Result<Vec<_>, _>>()?;
            validate_keyed_order(desired.iter().map(|(key, _)| key.clone()))
                .map_err(keyed_error)?;

            let document = self
                .root
                .owner_document()
                .ok_or_else(|| JsValue::from_str("keyed DOM root has no document"))?;
            let mut created = HashMap::new();
            for (key, item) in &desired {
                if self.entries.contains_key(key) {
                    continue;
                }
                let entry = create(&document, key, item)?;
                if entry.element.parent_node().is_some() {
                    return Err(JsValue::from_str(
                        "new keyed DOM entry must be detached before reconciliation",
                    ));
                }
                created.insert(key.clone(), entry);
            }

            for (key, item) in &desired {
                if let Some(entry) = self.entries.get_mut(key) {
                    let (element, state) = entry.parts_mut();
                    update(key, element, state, item)?;
                }
            }

            let keep = desired
                .iter()
                .map(|(key, _)| key.clone())
                .collect::<HashSet<_>>();
            let stale = self
                .entries
                .keys()
                .filter(|key| !keep.contains(*key))
                .cloned()
                .collect::<Vec<_>>();
            for key in &stale {
                let entry = self
                    .entries
                    .get(key)
                    .ok_or_else(|| JsValue::from_str("stale keyed DOM entry missing"))?;
                self.root.remove_child(entry.element())?;
            }

            let mut stats = ReconcileStats {
                retained: desired.len().saturating_sub(created.len()),
                inserted: created.len(),
                removed: stale.len(),
                moved: 0,
            };
            for (index, (key, _)) in desired.iter().enumerate() {
                let was_existing = self.entries.contains_key(key);
                let element = self
                    .entries
                    .get(key)
                    .or_else(|| created.get(key))
                    .map(KeyedEntry::element)
                    .ok_or_else(|| JsValue::from_str("desired keyed DOM entry missing"))?;
                let current = self.root.children().item(index as u32);
                if !current
                    .as_ref()
                    .is_some_and(|node| node.is_same_node(Some(element)))
                {
                    let reference: Option<Node> = current.map(Into::into);
                    self.root.insert_before(element, reference.as_ref())?;
                    if was_existing {
                        stats.moved += 1;
                    }
                }
            }

            for key in stale {
                self.entries.remove(&key);
            }
            self.entries.extend(created);
            Ok(stats)
        }

        pub fn dispose(&mut self) -> bool {
            if self.disposed {
                return false;
            }
            self.disposed = true;
            self.entries.clear();
            true
        }
    }

    impl<State> Drop for KeyedChildren<State> {
        fn drop(&mut self) {
            if !self.disposed {
                self.disposed = true;
                self.entries.clear();
            }
        }
    }

    fn direct_element_children(root: &Element) -> Result<Vec<Element>, JsValue> {
        let mut elements = Vec::new();
        let mut child = root.first_child();
        while let Some(node) = child {
            child = node.next_sibling();
            if node.node_type() != Node::ELEMENT_NODE {
                return Err(JsValue::from_str(
                    "keyed DOM root contains a non-element direct child",
                ));
            }
            elements.push(
                node.dyn_into::<Element>()
                    .map_err(|_| JsValue::from_str("keyed DOM child conversion failed"))?,
            );
        }
        Ok(elements)
    }
}

#[cfg(target_arch = "wasm32")]
pub use browser::{DomEventListener, KeyedChildren, KeyedEntry, ReconcileStats};

#[cfg(test)]
mod tests {
    use super::{KeyedOrderError, validate_keyed_order};
    use yetlet::view::binding::BindingKey;

    #[test]
    fn keyed_order_preserves_input_order_and_rejects_duplicates() {
        let order = validate_keyed_order([
            BindingKey::new("three").unwrap(),
            BindingKey::new("one").unwrap(),
            BindingKey::new("two").unwrap(),
        ])
        .unwrap();
        assert_eq!(
            order.iter().map(BindingKey::as_str).collect::<Vec<_>>(),
            ["three", "one", "two"]
        );
        assert_eq!(
            validate_keyed_order([
                BindingKey::new("one").unwrap(),
                BindingKey::new("one").unwrap(),
            ]),
            Err(KeyedOrderError::DuplicateKey("one".into()))
        );
    }
}
