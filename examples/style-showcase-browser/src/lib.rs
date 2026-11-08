//! One explicitly owned listener set; no hydration, CSS compilation or circuit.
#[cfg(target_arch = "wasm32")]
mod browser {
    use std::{cell::RefCell, rc::Rc};
    use wasm_bindgen::{JsCast, prelude::*};
    use web_sys::{Document, Element, MouseEvent};
    use yetlet_browser::dom_keyed::DomEventListener;
    use yetlet_style_showcase::{Selection, style_names::*};

    thread_local! { static LISTENERS: RefCell<Vec<DomEventListener>> = const { RefCell::new(Vec::new()) }; }

    fn render(document: &Document, root: &Element, selection: Selection) -> Result<(), JsValue> {
        root.set_class_name(&selection.classes());
        for (name, value) in [
            ("theme", selection.theme()),
            ("density", selection.density()),
            ("selected", selection.selected()),
        ] {
            root.set_attribute(&format!("data-{name}"), value)?;
        }
        let choices = root.query_selector_all("a[data-choice]")?;
        for index in 0..choices.length() {
            let link: Element = choices.item(index).unwrap().dyn_into()?;
            let kind = link.get_attribute("data-choice").unwrap();
            let value = link.get_attribute("data-value").unwrap();
            let selected = selection.value(&kind) == value;
            link.set_attribute("aria-current", if selected { "true" } else { "false" })?;
            link.set_attribute(
                "href",
                &selection.change(&kind, &value).href(
                    "/showcase/interactive",
                    if kind == "selected" {
                        "states"
                    } else {
                        "tokens"
                    },
                ),
            )?;
            if kind == "selected" {
                link.set_class_name(&format!(
                    "{CARD} {}",
                    if selected { SELECTED } else { UNSELECTED }
                ));
                link.query_selector(".selection-label")?
                    .unwrap()
                    .set_text_content(Some(if selected {
                        "Selected"
                    } else {
                        "Select this example"
                    }));
            }
        }
        document
            .get_element_by_id("selection-status")
            .unwrap()
            .set_text_content(Some(if selection.second {
                "Current example: Optional enhancement."
            } else {
                "Current example: Native HTML."
            }));
        Ok(())
    }

    #[wasm_bindgen]
    pub fn enhance() -> Result<(), JsValue> {
        dispose();
        let document = web_sys::window().unwrap().document().unwrap();
        let root = document
            .get_element_by_id("showcase")
            .ok_or_else(|| JsValue::from_str("missing shared View"))?;
        let selection = Rc::new(RefCell::new(Selection::from_values(
            &root.get_attribute("data-theme").unwrap_or_default(),
            &root.get_attribute("data-density").unwrap_or_default(),
            &root.get_attribute("data-selected").unwrap_or_default(),
        )));
        let choices = root.query_selector_all("a[data-choice]")?;
        let mut listeners = Vec::new();
        for index in 0..choices.length() {
            let link: Element = choices.item(index).unwrap().dyn_into()?;
            let kind = link.get_attribute("data-choice").unwrap();
            let value = link.get_attribute("data-value").unwrap();
            let selection = selection.clone();
            let root = root.clone();
            let document = document.clone();
            listeners.push(DomEventListener::new(
                link.as_ref(),
                "click",
                move |event| {
                    let Some(mouse) = event.dyn_ref::<MouseEvent>() else {
                        return;
                    };
                    if mouse.button() != 0
                        || mouse.ctrl_key()
                        || mouse.meta_key()
                        || mouse.shift_key()
                        || mouse.alt_key()
                    {
                        return;
                    }
                    event.prevent_default();
                    let next = selection.borrow().change(&kind, &value);
                    if render(&document, &root, next).is_ok() {
                        *selection.borrow_mut() = next;
                    }
                },
            )?);
        }
        LISTENERS.with(|slot| *slot.borrow_mut() = listeners);
        root.set_attribute("data-enhanced", "true")?;
        document.get_element_by_id("enhancement-status").unwrap().set_text_content(Some("Rust/WASM enhancement active · theme, density and selection change locally. No server requests."));
        Ok(())
    }

    #[wasm_bindgen]
    pub fn dispose() {
        LISTENERS.with(|slot| slot.borrow_mut().clear());
        if let Some(document) = web_sys::window().and_then(|w| w.document())
            && let Some(root) = document.get_element_by_id("showcase")
        {
            let _ = root.remove_attribute("data-enhanced");
        }
    }
}
#[cfg(target_arch = "wasm32")]
pub use browser::{dispose, enhance};
