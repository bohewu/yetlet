use serde::Deserialize;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::{JsCast, prelude::*};
use yetlet::view::{Element, View};

#[derive(Deserialize)]
struct Entry {
    title: String,
    href: String,
    text: String,
}
struct Runtime {
    input: web_sys::HtmlInputElement,
    listener: Closure<dyn FnMut(web_sys::Event)>,
}
impl Drop for Runtime {
    fn drop(&mut self) {
        let _ = self
            .input
            .remove_event_listener_with_callback("input", self.listener.as_ref().unchecked_ref());
    }
}
thread_local! { static OWNER: RefCell<Option<Runtime>> = const { RefCell::new(None) }; }

#[wasm_bindgen]
pub fn activate(index: &str) -> Result<(), JsValue> {
    OWNER.with(|owner| {
        if owner.borrow().is_some() {
            return Err(JsValue::from_str("Search is already active"));
        }
        let entries: Vec<Entry> = serde_json::from_str(index)
            .map_err(|_| JsValue::from_str("Search index unavailable"))?;
        if entries.len() != 8 {
            return Err(JsValue::from_str("Unexpected chapter index"));
        }
        let document = web_sys::window()
            .and_then(|w| w.document())
            .ok_or_else(|| JsValue::from_str("Document unavailable"))?;
        let input: web_sys::HtmlInputElement = document
            .get_element_by_id("site-search-input")
            .ok_or_else(|| JsValue::from_str("Search input unavailable"))?
            .dyn_into()?;
        let results = document
            .get_element_by_id("site-search-results")
            .ok_or_else(|| JsValue::from_str("Search results unavailable"))?;
        let entries = Rc::new(entries);
        let observed = input.clone();
        let observed_results = results.clone();
        let observed_entries = entries.clone();
        let listener = Closure::wrap(Box::new(move |_: web_sys::Event| {
            render_results(&observed, &observed_results, &observed_entries);
        }) as Box<dyn FnMut(web_sys::Event)>);
        input.add_event_listener_with_callback("input", listener.as_ref().unchecked_ref())?;
        render_results(&input, &results, &entries);
        *owner.borrow_mut() = Some(Runtime { input, listener });
        Ok(())
    })
}
#[wasm_bindgen]
pub fn dispose() {
    OWNER.with(|owner| {
        owner.borrow_mut().take();
    });
}

fn render_results(
    input: &web_sys::HtmlInputElement,
    results: &web_sys::Element,
    entries: &[Entry],
) {
    let query = input
        .value()
        .chars()
        .take(100)
        .collect::<String>()
        .trim()
        .to_lowercase();
    if query.is_empty() {
        results.set_text_content(None);
        let _ = results.set_attribute("hidden", "");
        return;
    }
    let hits: Vec<_> = entries
        .iter()
        .filter(|e| {
            e.title.to_lowercase().contains(&query) || e.text.to_lowercase().contains(&query)
        })
        .collect();
    let view: View = if hits.is_empty() {
        Element::new("p")
            .child(View::text("No matching chapters."))
            .into()
    } else {
        Element::new("ul")
            .children(hits.into_iter().map(|e| {
                Element::new("li")
                    .child(
                        Element::new("a")
                            .attribute("href", &e.href)
                            .child(
                                Element::new("span")
                                    .attribute("class", "result-title")
                                    .child(View::text(&e.title)),
                            )
                            .child(
                                Element::new("span")
                                    .attribute("class", "result-excerpt")
                                    .child(View::text(
                                        e.text.chars().take(150).collect::<String>(),
                                    )),
                            ),
                    )
                    .into()
            }))
            .into()
    };
    if yetlet_browser::dom::mount(results, &view).is_err() {
        results.set_text_content(Some("Search unavailable. Reload to retry."));
    }
    if input.matches(":focus").unwrap_or(false) {
        let _ = results.remove_attribute("hidden");
    } else {
        let _ = results.set_attribute("hidden", "");
    }
}
