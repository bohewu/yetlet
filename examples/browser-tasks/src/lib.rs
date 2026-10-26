//! Browser entry point. Only the model is compiled for native focused tests.
pub mod model;

#[cfg(target_arch = "wasm32")]
mod browser {
    use super::model::{Command, Filter, Task, Tasks};
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };
    use wasm_bindgen::{JsCast, JsValue, prelude::wasm_bindgen};
    use web_sys::{Document, Element, HtmlInputElement};
    use yetlet_core::{Element as ViewElement, View, binding::BindingKey};
    use yetlet_dom_keyed::{DomEventListener, KeyedChildren, KeyedEntry};
    use yetlet_reactive::{GraphHandle, LocalGraph};

    type Handle = GraphHandle<Tasks, Command, String>;

    struct Runtime {
        _graph: LocalGraph<Tasks, Command, String>,
        _bindings: Vec<DomEventListener>,
    }
    thread_local! {
        static RUNTIME: RefCell<Option<Runtime>> = const { RefCell::new(None) };
    }

    struct Row {
        checkbox: HtmlInputElement,
        title: Element,
        edit: Element,
    }

    fn element(document: &Document, id: &str) -> Result<Element, JsValue> {
        document
            .get_element_by_id(id)
            .ok_or_else(|| JsValue::from_str("Missing task workspace element"))
    }
    fn input(document: &Document, id: &str) -> Result<HtmlInputElement, JsValue> {
        element(document, id)?
            .dyn_into()
            .map_err(|_| JsValue::from_str("Invalid task input"))
    }
    fn key(task: &Task) -> Result<BindingKey, JsValue> {
        BindingKey::new(task.id.to_string()).map_err(|_| JsValue::from_str("Invalid task identity"))
    }
    fn update_row(
        _key: &BindingKey,
        element: &Element,
        row: &mut Row,
        task: &Task,
    ) -> Result<(), JsValue> {
        row.title.set_text_content(Some(&task.title));
        row.checkbox.set_checked(task.completed);
        row.edit
            .set_attribute("aria-label", &format!("Edit {}", task.title))?;
        element.set_class_name(if task.completed {
            "task-row completed"
        } else {
            "task-row"
        });
        Ok(())
    }
    fn create_row(
        document: &Document,
        key: &BindingKey,
        task: &Task,
    ) -> Result<KeyedEntry<Row>, JsValue> {
        let host = document.create_element("div")?;
        let view: View = ViewElement::new("li")
            .attribute("class", "task-row")
            .child(
                ViewElement::new("label")
                    .attribute("class", "task-label")
                    .child(
                        ViewElement::new("input")
                            .attribute("type", "checkbox")
                            .attribute("id", format!("task-{}", task.id))
                            .attribute("data-id", task.id.to_string()),
                    )
                    .child(
                        ViewElement::new("span")
                            .attribute("class", "task-title")
                            .child(View::text(&task.title)),
                    ),
            )
            .child(
                ViewElement::new("button")
                    .attribute("type", "button")
                    .attribute("data-id", task.id.to_string())
                    .child(View::text("Edit")),
            )
            .into();
        yetlet_dom::mount(&host, &view)?;
        let element = host
            .first_element_child()
            .ok_or_else(|| JsValue::from_str("Missing task row"))?;
        let checkbox = element
            .query_selector("input")?
            .ok_or_else(|| JsValue::from_str("Missing checkbox"))?
            .dyn_into::<HtmlInputElement>()?;
        let title = element
            .query_selector("span")?
            .ok_or_else(|| JsValue::from_str("Missing title"))?;
        let edit = element
            .query_selector("button")?
            .ok_or_else(|| JsValue::from_str("Missing edit button"))?;
        let mut row = Row {
            checkbox,
            title,
            edit,
        };
        update_row(key, &element, &mut row, task)?;
        host.remove_child(&element)?;
        Ok(KeyedEntry::new(element, row))
    }

    fn dispatch(handle: &Handle, command: Command, feedback: &Element, success: &str) -> bool {
        match handle.dispatch(command) {
            Ok(_) => {
                feedback.set_text_content(Some(success));
                true
            }
            Err(error) => {
                feedback.set_text_content(Some(&error.to_string()));
                false
            }
        }
    }

    #[wasm_bindgen]
    pub fn mount_tasks() -> Result<(), JsValue> {
        RUNTIME.with(|slot| slot.borrow_mut().take());
        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| JsValue::from_str("Missing document"))?;
        let list = element(&document, "tasks")?;
        list.set_text_content(None);
        let mut rows = KeyedChildren::from_existing(list.clone(), vec![])?;
        let count = element(&document, "count")?;
        let empty = element(&document, "empty")?;
        let filters = [
            element(&document, "filter-all")?,
            element(&document, "filter-active")?,
            element(&document, "filter-completed")?,
        ];
        let initial = Tasks::default();
        let committed = Rc::new(RefCell::new(initial.clone()));
        let snapshot = committed.clone();
        let filter_buttons = filters.clone();
        let mut render = move |state: &Tasks| -> Result<(), String> {
            let visible = state.visible();
            rows.reconcile(&visible, key, update_row, create_row)
                .map_err(|error| format!("Unable to render tasks: {error:?}"))?;
            let active = state.items.iter().filter(|task| !task.completed).count();
            count.set_text_content(Some(&format!(
                "{} tasks · {active} active",
                state.items.len()
            )));
            empty
                .toggle_attribute_with_force("hidden", !visible.is_empty())
                .map_err(|_| "Unable to render empty state.")?;
            for (index, button) in filter_buttons.iter().enumerate() {
                let selected =
                    [Filter::All, Filter::Active, Filter::Completed][index] == state.filter;
                button
                    .set_attribute("aria-pressed", if selected { "true" } else { "false" })
                    .map_err(|_| "Unable to render filters.")?;
            }
            *snapshot.borrow_mut() = state.clone();
            Ok(())
        };
        render(&initial).map_err(|error| JsValue::from_str(&error))?;
        let graph = LocalGraph::new(initial, Tasks::apply, render);
        let handle = graph.handle();
        let feedback = element(&document, "feedback")?;
        let new_task = input(&document, "new-task")?;
        let edit_title = input(&document, "edit-title")?;
        let edit_form = element(&document, "edit-form")?;
        let editing = Rc::new(Cell::new(None::<u32>));
        let mut bindings = Vec::new();

        {
            let (handle, feedback, new_task) = (handle.clone(), feedback.clone(), new_task.clone());
            bindings.push(DomEventListener::new(
                element(&document, "add-form")?.as_ref(),
                "submit",
                move |event| {
                    event.prevent_default();
                    if dispatch(
                        &handle,
                        Command::Add(new_task.value()),
                        &feedback,
                        "Task added.",
                    ) {
                        new_task.set_value("");
                        let _ = new_task.focus();
                    }
                },
            )?);
        }
        for (button, filter) in filters
            .iter()
            .zip([Filter::All, Filter::Active, Filter::Completed])
        {
            let (handle, feedback) = (handle.clone(), feedback.clone());
            bindings.push(DomEventListener::new(
                button.as_ref(),
                "click",
                move |_| {
                    dispatch(&handle, Command::Filter(filter), &feedback, "");
                },
            )?);
        }
        {
            let (handle, feedback) = (handle.clone(), feedback.clone());
            bindings.push(DomEventListener::new(
                list.as_ref(),
                "change",
                move |event| {
                    if let Some(input) = event
                        .target()
                        .and_then(|target| target.dyn_into::<HtmlInputElement>().ok())
                        && let Some(id) = input
                            .get_attribute("data-id")
                            .and_then(|value| value.parse().ok())
                    {
                        dispatch(
                            &handle,
                            Command::Complete(id, input.checked()),
                            &feedback,
                            "Task updated.",
                        );
                    }
                },
            )?);
        }
        {
            let (editing, edit_form, edit_title, feedback) = (
                editing.clone(),
                edit_form.clone(),
                edit_title.clone(),
                feedback.clone(),
            );
            bindings.push(DomEventListener::new(
                list.as_ref(),
                "click",
                move |event| {
                    let button = event
                        .target()
                        .and_then(|target| target.dyn_into::<Element>().ok())
                        .and_then(|target| target.closest("button[data-id]").ok().flatten());
                    if let Some(id) = button
                        .and_then(|button| button.get_attribute("data-id"))
                        .and_then(|value| value.parse::<u32>().ok())
                        && let Some(task) =
                            committed.borrow().items.iter().find(|task| task.id == id)
                    {
                        editing.set(Some(id));
                        edit_title.set_value(&task.title);
                        let _ = edit_form.remove_attribute("hidden");
                        feedback.set_text_content(None);
                        let _ = edit_title.focus();
                    }
                },
            )?);
        }
        {
            let (editing, edit_form, edit_title, feedback, handle, new_task) = (
                editing.clone(),
                edit_form.clone(),
                edit_title.clone(),
                feedback.clone(),
                handle.clone(),
                new_task.clone(),
            );
            bindings.push(DomEventListener::new(
                edit_form.clone().as_ref(),
                "submit",
                move |event| {
                    event.prevent_default();
                    if let Some(id) = editing.get()
                        && dispatch(
                            &handle,
                            Command::Rename(id, edit_title.value()),
                            &feedback,
                            "Task saved.",
                        )
                    {
                        editing.set(None);
                        let _ = edit_form.set_attribute("hidden", "");
                        let _ = new_task.focus();
                    }
                },
            )?);
        }
        {
            let new_task = new_task.clone();
            bindings.push(DomEventListener::new(
                element(&document, "cancel-edit")?.as_ref(),
                "click",
                move |_| {
                    editing.set(None);
                    let _ = edit_form.set_attribute("hidden", "");
                    let _ = new_task.focus();
                },
            )?);
        }
        // Focus/key input and edit values stay native; no controlled-value loop.
        // Retained rows use keyed ownership, rather than replacing active controls.
        element(&document, "workspace")?.remove_attribute("disabled")?;
        element(&document, "runtime-status")?.set_attribute("hidden", "")?;
        RUNTIME.with(|slot| {
            *slot.borrow_mut() = Some(Runtime {
                _graph: graph,
                _bindings: bindings,
            })
        });
        Ok(())
    }
    /// Retire app-owned listeners/graph; late bootstrap must not mount afterward.
    #[wasm_bindgen]
    pub fn dispose_tasks() {
        RUNTIME.with(|slot| slot.borrow_mut().take());
    }
}
