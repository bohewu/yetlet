#![cfg_attr(not(any(test, target_arch = "wasm32")), allow(dead_code))]

//! Isolated M7a render-strategy benchmark.
//!
//! This crate is a measurement fixture, not a stable renderer API. It compares
//! the current Yetlet owned-subtree replacement behavior with three bounded
//! alternatives under one deterministic workload. Production Todo/Search
//! bundles do not depend on this crate.

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
struct RowModel {
    key: u32,
    completed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BenchOperation {
    Toggle(u32),
    Append(RowModel),
    Remove(u32),
    MoveToEnd(u32),
}

fn initial_model(rows: u32) -> Vec<RowModel> {
    (1..=rows)
        .map(|key| RowModel {
            key,
            completed: false,
        })
        .collect()
}

fn row_label(row: &RowModel) -> String {
    format!(
        "Row {:03}: {}",
        row.key,
        if row.completed { "complete" } else { "active" }
    )
}

fn deterministic_toggles(rows: u32, updates: u32, seed: u32) -> Vec<BenchOperation> {
    let mut value = u64::from(seed).max(1);
    (0..updates)
        .map(|_| {
            value = value
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            BenchOperation::Toggle((value % u64::from(rows)) as u32 + 1)
        })
        .collect()
}

fn structural_operations(rows: u32) -> [BenchOperation; 3] {
    [
        BenchOperation::Append(RowModel {
            key: rows + 1,
            completed: true,
        }),
        BenchOperation::Remove(1),
        BenchOperation::MoveToEnd(2),
    ]
}

fn apply_model(model: &mut Vec<RowModel>, operation: &BenchOperation) {
    match operation {
        BenchOperation::Toggle(key) => {
            let row = model
                .iter_mut()
                .find(|row| row.key == *key)
                .expect("generated key must exist");
            row.completed = !row.completed;
        }
        BenchOperation::Append(row) => model.push(row.clone()),
        BenchOperation::Remove(key) => {
            let index = model
                .iter()
                .position(|row| row.key == *key)
                .expect("removed key must exist");
            model.remove(index);
        }
        BenchOperation::MoveToEnd(key) => {
            let index = model
                .iter()
                .position(|row| row.key == *key)
                .expect("moved key must exist");
            let row = model.remove(index);
            model.push(row);
        }
    }
}

fn model_checksum(model: &[RowModel]) -> u64 {
    let mut value = 0xcbf2_9ce4_8422_2325_u64;
    for row in model {
        for byte in format!("{}|{}|{};", row.key, row.completed, row_label(row)).bytes() {
            value ^= u64::from(byte);
            value = value.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    value
}

#[cfg(target_arch = "wasm32")]
#[derive(Debug, Default, Clone, Copy, Serialize)]
struct DomOperations {
    node_creations: u64,
    text_mutations: u64,
    attribute_mutations: u64,
    child_insertions: u64,
    child_removals: u64,
    child_moves: u64,
}

#[cfg(target_arch = "wasm32")]
impl DomOperations {
    fn add(&mut self, other: Self) {
        self.node_creations += other.node_creations;
        self.text_mutations += other.text_mutations;
        self.attribute_mutations += other.attribute_mutations;
        self.child_insertions += other.child_insertions;
        self.child_removals += other.child_removals;
        self.child_moves += other.child_moves;
    }
}

#[derive(Debug, Serialize)]
struct TimingSummary {
    samples: usize,
    median_ms: f64,
    p95_ms: f64,
    min_ms: f64,
    max_ms: f64,
}

fn timing_summary(mut values: Vec<f64>) -> TimingSummary {
    values.sort_by(|left, right| left.total_cmp(right));
    let samples = values.len();
    let p95_index = ((samples as f64 * 0.95).ceil() as usize)
        .saturating_sub(1)
        .min(samples - 1);
    TimingSummary {
        samples,
        median_ms: round_micros(if samples.is_multiple_of(2) {
            (values[samples / 2 - 1] + values[samples / 2]) / 2.0
        } else {
            values[samples / 2]
        }),
        p95_ms: round_micros(values[p95_index]),
        min_ms: round_micros(values[0]),
        max_ms: round_micros(values[samples - 1]),
    }
}

fn round_micros(value: f64) -> f64 {
    (value * 1_000.0).round() / 1_000.0
}

#[cfg(target_arch = "wasm32")]
mod browser {
    use std::collections::HashMap;

    use js_sys::Array;
    use serde::Serialize;
    use wasm_bindgen::{JsCast, JsValue, prelude::wasm_bindgen};
    use web_sys::{Document, Element, HtmlInputElement, Node, Text};
    use yetlet_core::{Attribute, Element as ViewElement, View};

    use super::{
        BenchOperation, DomOperations, RowModel, apply_model, deterministic_toggles, initial_model,
        model_checksum, round_micros, row_label, structural_operations, timing_summary,
    };

    const WARMUP_UPDATES: u32 = 50;

    #[derive(Clone, Copy, Debug)]
    enum StrategyKind {
        SubtreeReplacement,
        ViewTreePatch,
        KeyedDirect,
        StaticSlots,
    }

    impl StrategyKind {
        const ALL: [Self; 4] = [
            Self::SubtreeReplacement,
            Self::ViewTreePatch,
            Self::KeyedDirect,
            Self::StaticSlots,
        ];

        fn name(self) -> &'static str {
            match self {
                Self::SubtreeReplacement => "subtree_replacement",
                Self::ViewTreePatch => "view_tree_patch",
                Self::KeyedDirect => "keyed_direct_dom",
                Self::StaticSlots => "static_template_slots",
            }
        }
    }

    #[derive(Debug, Serialize)]
    struct BenchmarkConfig {
        rows: u32,
        updates_per_round: u32,
        rounds: u32,
        warmup_rounds: u32,
        warmup_updates: u32,
        seed: u32,
        order_policy: &'static str,
        structural_operations: [&'static str; 3],
    }

    #[derive(Debug, Serialize)]
    struct StrategyReport {
        strategy: &'static str,
        setup: super::TimingSummary,
        update: super::TimingSummary,
        structural: super::TimingSummary,
        total_round: super::TimingSummary,
        setup_dom_operations: DomOperations,
        update_dom_operations: DomOperations,
        structural_dom_operations: DomOperations,
        outside_focus_preserved_all_rounds: bool,
        inside_input_identity_preserved_all_rounds: bool,
        inside_focus_preserved_all_rounds: bool,
        untouched_row_identity_preserved_all_rounds: bool,
        final_state_equivalent_all_rounds: bool,
        final_state_checksums: Vec<String>,
    }

    #[derive(Debug, Serialize)]
    struct BenchmarkReport {
        schema_version: u32,
        scope: &'static str,
        config: BenchmarkConfig,
        strategy_order_by_round: Vec<Vec<&'static str>>,
        strategies: Vec<StrategyReport>,
        limitations: [&'static str; 7],
    }

    #[derive(Default)]
    struct StrategyAccumulator {
        setup: Vec<f64>,
        update: Vec<f64>,
        structural: Vec<f64>,
        total: Vec<f64>,
        setup_ops: DomOperations,
        update_ops: DomOperations,
        structural_ops: DomOperations,
        outside_focus: bool,
        inside_identity: bool,
        inside_focus: bool,
        row_identity: bool,
        equivalent: bool,
        final_checksums: Vec<u64>,
    }

    impl StrategyAccumulator {
        fn new() -> Self {
            Self {
                outside_focus: true,
                inside_identity: true,
                inside_focus: true,
                row_identity: true,
                equivalent: true,
                ..Self::default()
            }
        }

        fn add_round(&mut self, result: RoundResult) {
            self.setup.push(result.setup_ms);
            self.update.extend(result.update_times);
            self.structural.extend(result.structural_times);
            self.total.push(result.total_ms);
            self.setup_ops.add(result.setup_ops);
            self.update_ops.add(result.update_ops);
            self.structural_ops.add(result.structural_ops);
            self.outside_focus &= result.outside_focus;
            self.inside_identity &= result.inside_identity;
            self.inside_focus &= result.inside_focus;
            self.row_identity &= result.row_identity;
            self.equivalent &= result.equivalent;
            self.final_checksums.push(result.final_checksum);
        }

        fn finish(self, strategy: StrategyKind) -> StrategyReport {
            StrategyReport {
                strategy: strategy.name(),
                setup: timing_summary(self.setup),
                update: timing_summary(self.update),
                structural: timing_summary(self.structural),
                total_round: timing_summary(self.total),
                setup_dom_operations: self.setup_ops,
                update_dom_operations: self.update_ops,
                structural_dom_operations: self.structural_ops,
                outside_focus_preserved_all_rounds: self.outside_focus,
                inside_input_identity_preserved_all_rounds: self.inside_identity,
                inside_focus_preserved_all_rounds: self.inside_focus,
                untouched_row_identity_preserved_all_rounds: self.row_identity,
                final_state_equivalent_all_rounds: self.equivalent,
                final_state_checksums: self
                    .final_checksums
                    .into_iter()
                    .map(|checksum| format!("{checksum:016x}"))
                    .collect(),
            }
        }
    }

    struct RoundResult {
        setup_ms: f64,
        update_times: Vec<f64>,
        structural_times: Vec<f64>,
        total_ms: f64,
        setup_ops: DomOperations,
        update_ops: DomOperations,
        structural_ops: DomOperations,
        outside_focus: bool,
        inside_identity: bool,
        inside_focus: bool,
        row_identity: bool,
        equivalent: bool,
        final_checksum: u64,
    }

    struct RowSlot {
        element: Element,
        text: Text,
    }

    enum StrategyState {
        Subtree {
            target: Element,
        },
        TreePatch {
            target: Element,
            current: View,
        },
        Keyed {
            target: Element,
            list: Element,
            rows: HashMap<u32, Element>,
        },
        StaticSlots {
            target: Element,
            list: Element,
            rows: HashMap<u32, RowSlot>,
            prototype: Element,
        },
    }

    fn document() -> Result<Document, JsValue> {
        web_sys::window()
            .ok_or_else(|| JsValue::from_str("window unavailable"))?
            .document()
            .ok_or_else(|| JsValue::from_str("document unavailable"))
    }

    fn performance_now() -> Result<f64, JsValue> {
        Ok(web_sys::window()
            .ok_or_else(|| JsValue::from_str("window unavailable"))?
            .performance()
            .ok_or_else(|| JsValue::from_str("performance API unavailable"))?
            .now())
    }

    fn full_view(rows: &[RowModel]) -> View {
        let children = rows.iter().map(|row| {
            View::from(
                ViewElement::new("li")
                    .attribute("data-id", row.key.to_string())
                    .attribute("data-completed", row.completed.to_string())
                    .attribute(
                        "class",
                        if row.completed {
                            "bench-row is-complete"
                        } else {
                            "bench-row is-active"
                        },
                    )
                    .child(View::text(row_label(row))),
            )
        });
        ViewElement::new("section")
            .attribute("id", "bench-owned")
            .child(
                ViewElement::new("input")
                    .attribute("id", "inside-focus")
                    .attribute("type", "text"),
            )
            .child(
                ViewElement::new("ul")
                    .attribute("id", "bench-list")
                    .children(children),
            )
            .into()
    }

    fn count_view_nodes(view: &View) -> u64 {
        match view {
            View::Text(_) => 1,
            View::Element(element) => {
                1 + element
                    .child_views()
                    .iter()
                    .map(count_view_nodes)
                    .sum::<u64>()
            }
        }
    }

    fn create_node(
        document: &Document,
        view: &View,
        operations: &mut DomOperations,
    ) -> Result<Node, JsValue> {
        operations.node_creations += 1;
        match view {
            View::Text(value) => Ok(document.create_text_node(value).into()),
            View::Element(element) => {
                let node = document.create_element(element.tag())?;
                for attribute in element.attributes() {
                    node.set_attribute(attribute.name, &attribute.value)?;
                }
                for child in element.child_views() {
                    node.append_child(&create_node(document, child, operations)?)?;
                    operations.child_insertions += 1;
                }
                Ok(node.into())
            }
        }
    }

    fn attribute_value<'a>(attributes: &'a [Attribute], name: &str) -> Option<&'a str> {
        attributes
            .iter()
            .find(|attribute| attribute.name == name)
            .map(|attribute| attribute.value.as_str())
    }

    fn patch_node(
        document: &Document,
        parent: &Node,
        dom: &Node,
        old: &View,
        new: &View,
        operations: &mut DomOperations,
    ) -> Result<Node, JsValue> {
        match (old, new) {
            (View::Text(old_text), View::Text(new_text)) => {
                if old_text != new_text {
                    dom.set_node_value(Some(new_text));
                    operations.text_mutations += 1;
                }
                Ok(dom.clone())
            }
            (View::Element(old_element), View::Element(new_element))
                if old_element.tag() == new_element.tag() =>
            {
                let element: Element = dom.clone().dyn_into()?;
                for old_attribute in old_element.attributes() {
                    if attribute_value(new_element.attributes(), old_attribute.name).is_none() {
                        element.remove_attribute(old_attribute.name)?;
                        operations.attribute_mutations += 1;
                    }
                }
                for new_attribute in new_element.attributes() {
                    if attribute_value(old_element.attributes(), new_attribute.name)
                        != Some(new_attribute.value.as_str())
                    {
                        element.set_attribute(new_attribute.name, &new_attribute.value)?;
                        operations.attribute_mutations += 1;
                    }
                }

                let old_children = old_element.child_views();
                let new_children = new_element.child_views();
                let common = old_children.len().min(new_children.len());
                for index in 0..common {
                    let child = element
                        .child_nodes()
                        .item(index as u32)
                        .ok_or_else(|| JsValue::from_str("missing child during patch"))?;
                    patch_node(
                        document,
                        &element,
                        &child,
                        &old_children[index],
                        &new_children[index],
                        operations,
                    )?;
                }
                for child in &new_children[common..] {
                    element.append_child(&create_node(document, child, operations)?)?;
                    operations.child_insertions += 1;
                }
                while element.child_nodes().length() > new_children.len() as u32 {
                    let child = element
                        .last_child()
                        .ok_or_else(|| JsValue::from_str("missing removable child"))?;
                    element.remove_child(&child)?;
                    operations.child_removals += 1;
                }
                Ok(element.into())
            }
            _ => {
                let replacement = create_node(document, new, operations)?;
                parent.replace_child(&replacement, dom)?;
                operations.child_removals += 1;
                operations.child_insertions += 1;
                Ok(replacement)
            }
        }
    }

    fn append_new_row(
        document: &Document,
        list: &Element,
        row: &RowModel,
        operations: &mut DomOperations,
    ) -> Result<Element, JsValue> {
        let element = document.create_element("li")?;
        operations.node_creations += 1;
        element.set_attribute("data-id", &row.key.to_string())?;
        element.set_attribute("data-completed", &row.completed.to_string())?;
        element.set_attribute(
            "class",
            if row.completed {
                "bench-row is-complete"
            } else {
                "bench-row is-active"
            },
        )?;
        let text = document.create_text_node(&row_label(row));
        operations.node_creations += 1;
        element.append_child(&text)?;
        operations.child_insertions += 1;
        list.append_child(&element)?;
        operations.child_insertions += 1;
        Ok(element)
    }

    fn set_row(
        element: &Element,
        row: &RowModel,
        operations: &mut DomOperations,
    ) -> Result<(), JsValue> {
        let completed = row.completed.to_string();
        if element.get_attribute("data-completed").as_deref() != Some(&completed) {
            element.set_attribute("data-completed", &completed)?;
            operations.attribute_mutations += 1;
        }
        let class_name = if row.completed {
            "bench-row is-complete"
        } else {
            "bench-row is-active"
        };
        if element.get_attribute("class").as_deref() != Some(class_name) {
            element.set_attribute("class", class_name)?;
            operations.attribute_mutations += 1;
        }
        let text = element
            .first_child()
            .ok_or_else(|| JsValue::from_str("row text missing"))?;
        let label = row_label(row);
        if text.node_value().as_deref() != Some(&label) {
            text.set_node_value(Some(&label));
            operations.text_mutations += 1;
        }
        Ok(())
    }

    fn create_direct_shell(
        document: &Document,
        target: &Element,
        rows: &[RowModel],
        operations: &mut DomOperations,
    ) -> Result<(Element, HashMap<u32, Element>), JsValue> {
        let section = document.create_element("section")?;
        let input = document.create_element("input")?;
        let list = document.create_element("ul")?;
        operations.node_creations += 3;
        section.set_attribute("id", "bench-owned")?;
        input.set_attribute("id", "inside-focus")?;
        input.set_attribute("type", "text")?;
        list.set_attribute("id", "bench-list")?;
        section.append_child(&input)?;
        section.append_child(&list)?;
        operations.child_insertions += 2;
        let mut elements = HashMap::new();
        for row in rows {
            elements.insert(row.key, append_new_row(document, &list, row, operations)?);
        }
        target.append_child(&section)?;
        operations.child_insertions += 1;
        Ok((list, elements))
    }

    fn create_slot_shell(
        document: &Document,
        target: &Element,
        rows: &[RowModel],
        operations: &mut DomOperations,
    ) -> Result<(Element, HashMap<u32, RowSlot>, Element), JsValue> {
        let section = document.create_element("section")?;
        let input = document.create_element("input")?;
        let list = document.create_element("ul")?;
        let prototype = document.create_element("li")?;
        let prototype_text = document.create_text_node("");
        operations.node_creations += 5;
        section.set_attribute("id", "bench-owned")?;
        input.set_attribute("id", "inside-focus")?;
        input.set_attribute("type", "text")?;
        list.set_attribute("id", "bench-list")?;
        prototype.append_child(&prototype_text)?;
        operations.child_insertions += 1;
        section.append_child(&input)?;
        section.append_child(&list)?;
        operations.child_insertions += 2;
        let mut slots = HashMap::new();
        for row in rows {
            let clone: Element = prototype.clone_node_with_deep(true)?.dyn_into()?;
            operations.node_creations += 2;
            clone.set_attribute("data-id", &row.key.to_string())?;
            clone.set_attribute("data-completed", &row.completed.to_string())?;
            clone.set_attribute(
                "class",
                if row.completed {
                    "bench-row is-complete"
                } else {
                    "bench-row is-active"
                },
            )?;
            let text: Text = clone
                .first_child()
                .ok_or_else(|| JsValue::from_str("slot text missing"))?
                .dyn_into()?;
            text.set_data(&row_label(row));
            list.append_child(&clone)?;
            operations.child_insertions += 1;
            slots.insert(
                row.key,
                RowSlot {
                    element: clone,
                    text,
                },
            );
        }
        target.append_child(&section)?;
        operations.child_insertions += 1;
        Ok((list, slots, prototype))
    }

    fn setup_strategy(
        kind: StrategyKind,
        document: &Document,
        target: Element,
        model: &[RowModel],
        operations: &mut DomOperations,
    ) -> Result<StrategyState, JsValue> {
        match kind {
            StrategyKind::SubtreeReplacement => {
                let view = full_view(model);
                let nodes = count_view_nodes(&view);
                yetlet_dom::mount(&target, &view)?;
                operations.node_creations += nodes;
                operations.child_insertions += nodes;
                Ok(StrategyState::Subtree { target })
            }
            StrategyKind::ViewTreePatch => {
                let view = full_view(model);
                yetlet_dom::validate(&view)
                    .map_err(|error| JsValue::from_str(&error.to_string()))?;
                let node = create_node(document, &view, operations)?;
                target.append_child(&node)?;
                operations.child_insertions += 1;
                Ok(StrategyState::TreePatch {
                    target,
                    current: view,
                })
            }
            StrategyKind::KeyedDirect => {
                let (list, rows) = create_direct_shell(document, &target, model, operations)?;
                Ok(StrategyState::Keyed { target, list, rows })
            }
            StrategyKind::StaticSlots => {
                let (list, rows, prototype) =
                    create_slot_shell(document, &target, model, operations)?;
                Ok(StrategyState::StaticSlots {
                    target,
                    list,
                    rows,
                    prototype,
                })
            }
        }
    }

    impl StrategyState {
        fn target(&self) -> &Element {
            match self {
                Self::Subtree { target }
                | Self::TreePatch { target, .. }
                | Self::Keyed { target, .. }
                | Self::StaticSlots { target, .. } => target,
            }
        }

        fn apply(
            &mut self,
            document: &Document,
            operation: &BenchOperation,
            model: &[RowModel],
            operations: &mut DomOperations,
        ) -> Result<(), JsValue> {
            match self {
                Self::Subtree { target } => {
                    let view = full_view(model);
                    let nodes = count_view_nodes(&view);
                    if target.first_child().is_some() {
                        operations.child_removals += 1;
                    }
                    yetlet_dom::mount(target, &view)?;
                    operations.node_creations += nodes;
                    operations.child_insertions += nodes;
                }
                Self::TreePatch { target, current } => {
                    let next = full_view(model);
                    yetlet_dom::validate(&next)
                        .map_err(|error| JsValue::from_str(&error.to_string()))?;
                    let root = target
                        .first_child()
                        .ok_or_else(|| JsValue::from_str("tree root missing"))?;
                    patch_node(document, target, &root, current, &next, operations)?;
                    *current = next;
                }
                Self::Keyed { list, rows, .. } => match operation {
                    BenchOperation::Toggle(key) => {
                        let row = model
                            .iter()
                            .find(|row| row.key == *key)
                            .ok_or_else(|| JsValue::from_str("toggled row missing"))?;
                        set_row(
                            rows.get(key)
                                .ok_or_else(|| JsValue::from_str("keyed element missing"))?,
                            row,
                            operations,
                        )?;
                    }
                    BenchOperation::Append(row) => {
                        rows.insert(row.key, append_new_row(document, list, row, operations)?);
                    }
                    BenchOperation::Remove(key) => {
                        let element = rows
                            .remove(key)
                            .ok_or_else(|| JsValue::from_str("removed element missing"))?;
                        list.remove_child(&element)?;
                        operations.child_removals += 1;
                    }
                    BenchOperation::MoveToEnd(key) => {
                        let element = rows
                            .get(key)
                            .ok_or_else(|| JsValue::from_str("moved element missing"))?;
                        list.append_child(element)?;
                        operations.child_moves += 1;
                    }
                },
                Self::StaticSlots {
                    list,
                    rows,
                    prototype,
                    ..
                } => match operation {
                    BenchOperation::Toggle(key) => {
                        let row = model
                            .iter()
                            .find(|row| row.key == *key)
                            .ok_or_else(|| JsValue::from_str("slot row missing"))?;
                        let slot = rows
                            .get(key)
                            .ok_or_else(|| JsValue::from_str("slot missing"))?;
                        let completed = row.completed.to_string();
                        if slot.element.get_attribute("data-completed").as_deref()
                            != Some(&completed)
                        {
                            slot.element.set_attribute("data-completed", &completed)?;
                            operations.attribute_mutations += 1;
                        }
                        let class_name = if row.completed {
                            "bench-row is-complete"
                        } else {
                            "bench-row is-active"
                        };
                        if slot.element.get_attribute("class").as_deref() != Some(class_name) {
                            slot.element.set_attribute("class", class_name)?;
                            operations.attribute_mutations += 1;
                        }
                        let label = row_label(row);
                        if slot.text.data() != label {
                            slot.text.set_data(&label);
                            operations.text_mutations += 1;
                        }
                    }
                    BenchOperation::Append(row) => {
                        let element: Element = prototype.clone_node_with_deep(true)?.dyn_into()?;
                        operations.node_creations += 2;
                        element.set_attribute("data-id", &row.key.to_string())?;
                        element.set_attribute("data-completed", &row.completed.to_string())?;
                        element.set_attribute(
                            "class",
                            if row.completed {
                                "bench-row is-complete"
                            } else {
                                "bench-row is-active"
                            },
                        )?;
                        let text: Text = element
                            .first_child()
                            .ok_or_else(|| JsValue::from_str("new slot text missing"))?
                            .dyn_into()?;
                        text.set_data(&row_label(row));
                        list.append_child(&element)?;
                        operations.child_insertions += 1;
                        rows.insert(row.key, RowSlot { element, text });
                    }
                    BenchOperation::Remove(key) => {
                        let slot = rows
                            .remove(key)
                            .ok_or_else(|| JsValue::from_str("removed slot missing"))?;
                        list.remove_child(&slot.element)?;
                        operations.child_removals += 1;
                    }
                    BenchOperation::MoveToEnd(key) => {
                        let slot = rows
                            .get(key)
                            .ok_or_else(|| JsValue::from_str("moved slot missing"))?;
                        list.append_child(&slot.element)?;
                        operations.child_moves += 1;
                    }
                },
            }
            Ok(())
        }
    }

    fn query_input(target: &Element) -> Result<HtmlInputElement, JsValue> {
        target
            .query_selector("#inside-focus")?
            .ok_or_else(|| JsValue::from_str("inside input missing"))?
            .dyn_into()
            .map_err(|_| JsValue::from_str("inside input has wrong type"))
    }

    fn query_row(target: &Element, key: u32) -> Result<Node, JsValue> {
        Ok(target
            .query_selector(&format!("[data-id=\"{key}\"]"))?
            .ok_or_else(|| JsValue::from_str("row missing"))?
            .into())
    }

    fn dom_checksum(target: &Element) -> Result<u64, JsValue> {
        let rows = target.query_selector_all("#bench-list li")?;
        let mut value = 0xcbf2_9ce4_8422_2325_u64;
        for index in 0..rows.length() {
            let element: Element = rows
                .item(index)
                .ok_or_else(|| JsValue::from_str("checksum row missing"))?
                .dyn_into()?;
            let text = format!(
                "{}|{}|{};",
                element.get_attribute("data-id").unwrap_or_default(),
                element.get_attribute("data-completed").unwrap_or_default(),
                element.text_content().unwrap_or_default()
            );
            for byte in text.bytes() {
                value ^= u64::from(byte);
                value = value.wrapping_mul(0x0000_0100_0000_01b3);
            }
        }
        Ok(value)
    }

    fn run_round(
        kind: StrategyKind,
        document: &Document,
        root: &Element,
        rows: u32,
        toggle_operations: &[BenchOperation],
    ) -> Result<RoundResult, JsValue> {
        let sandbox = document.create_element("div")?;
        let outside: HtmlInputElement = document.create_element("input")?.dyn_into()?;
        let target = document.create_element("div")?;
        sandbox.append_child(&outside)?;
        sandbox.append_child(&target)?;
        root.append_child(&sandbox)?;

        let mut model = initial_model(rows);
        let mut setup_ops = DomOperations::default();
        let setup_start = performance_now()?;
        let mut state = setup_strategy(kind, document, target, &model, &mut setup_ops)?;
        let setup_ms = performance_now()? - setup_start;

        // Unmeasured focus check: apply and revert the same deterministic toggle.
        outside.focus()?;
        let focus_operation = BenchOperation::Toggle(1);
        let mut ignored = DomOperations::default();
        apply_model(&mut model, &focus_operation);
        state.apply(document, &focus_operation, &model, &mut ignored)?;
        apply_model(&mut model, &focus_operation);
        state.apply(document, &focus_operation, &model, &mut ignored)?;
        let outside_focus = document
            .active_element()
            .is_some_and(|active| active.is_same_node(Some(&outside)));

        let input_before = query_input(state.target())?;
        input_before.set_value("focus sentinel");
        input_before.focus()?;
        let row_before = query_row(state.target(), 2)?;

        let total_start = performance_now()?;
        let mut update_times = Vec::with_capacity(toggle_operations.len());
        let mut update_ops = DomOperations::default();
        for operation in toggle_operations {
            apply_model(&mut model, operation);
            let start = performance_now()?;
            state.apply(document, operation, &model, &mut update_ops)?;
            update_times.push(performance_now()? - start);
        }

        let input_after_updates = query_input(state.target())?;
        let inside_identity = input_after_updates.is_same_node(Some(&input_before));
        let inside_focus = document
            .active_element()
            .is_some_and(|active| active.is_same_node(Some(&input_before)));

        let mut structural_times = Vec::with_capacity(3);
        let mut structural_ops = DomOperations::default();
        for operation in structural_operations(rows) {
            apply_model(&mut model, &operation);
            let start = performance_now()?;
            state.apply(document, &operation, &model, &mut structural_ops)?;
            structural_times.push(performance_now()? - start);
        }
        let total_ms = performance_now()? - total_start + setup_ms;

        let row_after = query_row(state.target(), 2)?;
        let row_identity = row_after.is_same_node(Some(&row_before));
        let expected_checksum = model_checksum(&model);
        let actual_checksum = dom_checksum(state.target())?;
        let equivalent = actual_checksum == expected_checksum;

        root.remove_child(&sandbox)?;
        Ok(RoundResult {
            setup_ms: round_micros(setup_ms),
            update_times,
            structural_times,
            total_ms: round_micros(total_ms),
            setup_ops,
            update_ops,
            structural_ops,
            outside_focus,
            inside_identity,
            inside_focus,
            row_identity,
            equivalent,
            final_checksum: actual_checksum,
        })
    }

    fn rotated_order(round: u32, seed: u32) -> [StrategyKind; 4] {
        let mut order = StrategyKind::ALL;
        let offset = ((round + seed) % StrategyKind::ALL.len() as u32) as usize;
        order.rotate_left(offset);
        order
    }

    /// Run the bounded M7a benchmark and return a JSON report.
    ///
    /// The function is synchronous so one browser call measures each strategy
    /// without per-update JS/WASM boundary crossings.
    #[wasm_bindgen]
    pub fn run_benchmark(
        rows: u32,
        updates_per_round: u32,
        rounds: u32,
        seed: u32,
    ) -> Result<String, JsValue> {
        if !(3..=1_000).contains(&rows) {
            return Err(JsValue::from_str("rows must be 3..=1000"));
        }
        if !(1..=10_000).contains(&updates_per_round) {
            return Err(JsValue::from_str("updates must be 1..=10000"));
        }
        if !(1..=30).contains(&rounds) {
            return Err(JsValue::from_str("rounds must be 1..=30"));
        }

        let document = document()?;
        let root = document
            .get_element_by_id("bench-root")
            .ok_or_else(|| JsValue::from_str("missing #bench-root"))?;
        root.set_text_content(None);

        // One bounded warm-up per strategy is intentionally excluded.
        let warmup = deterministic_toggles(rows, updates_per_round.min(WARMUP_UPDATES), seed);
        for strategy in StrategyKind::ALL {
            run_round(strategy, &document, &root, rows, &warmup)?;
        }

        let mut accumulators: HashMap<&'static str, StrategyAccumulator> = StrategyKind::ALL
            .into_iter()
            .map(|strategy| (strategy.name(), StrategyAccumulator::new()))
            .collect();
        let mut orders = Vec::with_capacity(rounds as usize);
        for round in 0..rounds {
            let operations = deterministic_toggles(
                rows,
                updates_per_round,
                seed.wrapping_add(round.wrapping_mul(0x9e37_79b9)),
            );
            let order = rotated_order(round, seed);
            orders.push(order.into_iter().map(StrategyKind::name).collect());
            for strategy in order {
                let result = run_round(strategy, &document, &root, rows, &operations)?;
                accumulators
                    .get_mut(strategy.name())
                    .expect("all strategies have accumulators")
                    .add_round(result);
            }
        }

        let strategies = StrategyKind::ALL
            .into_iter()
            .map(|strategy| {
                accumulators
                    .remove(strategy.name())
                    .expect("accumulator must exist")
                    .finish(strategy)
            })
            .collect();
        let report = BenchmarkReport {
            schema_version: 1,
            scope: "isolated_m7a_measurement_not_public_runtime_api",
            config: BenchmarkConfig {
                rows,
                updates_per_round,
                rounds,
                warmup_rounds: 1,
                warmup_updates: updates_per_round.min(WARMUP_UPDATES),
                seed,
                order_policy: "deterministic rotating strategy order per measured round",
                structural_operations: ["append", "remove key 1", "move key 2 to end"],
            },
            strategy_order_by_round: orders,
            strategies,
            limitations: [
                "One browser process and one host; no cross-device or concurrent-load claim.",
                "performance.now timings include Rust/WASM strategy code and browser DOM calls.",
                "DOM operation counts are instrumented API operations, not engine-internal work.",
                "The view-tree patch is a bounded positional prototype, not a stable VDOM API.",
                "The static-slot prototype uses cloneNode, not a compile-time Rust macro IR.",
                "No layout/paint memory profile or framework comparison is included.",
                "Results guide M7b research and do not select a production renderer alone.",
            ],
        };
        serde_json::to_string(&report)
            .map_err(|error| JsValue::from_str(&format!("serialize report: {error}")))
    }

    #[allow(dead_code)]
    fn _array_type_anchor(_: Array) {}
}

#[cfg(target_arch = "wasm32")]
pub use browser::run_benchmark;

#[cfg(test)]
mod tests {
    use super::{
        BenchOperation, apply_model, deterministic_toggles, initial_model, model_checksum,
        structural_operations, timing_summary,
    };

    #[test]
    fn deterministic_workload_is_repeatable_and_bounded() {
        let first = deterministic_toggles(200, 500, 7);
        let second = deterministic_toggles(200, 500, 7);
        assert_eq!(first, second);
        assert_eq!(first.len(), 500);
        assert!(first.iter().all(|operation| {
            matches!(operation, BenchOperation::Toggle(key) if (1..=200).contains(key))
        }));
    }

    #[test]
    fn structural_workload_has_one_equivalent_final_model() {
        let mut model = initial_model(5);
        for operation in deterministic_toggles(5, 10, 11) {
            apply_model(&mut model, &operation);
        }
        for operation in structural_operations(5) {
            apply_model(&mut model, &operation);
        }
        assert_eq!(model.len(), 5);
        assert_eq!(model.last().unwrap().key, 2);
        assert!(model.iter().any(|row| row.key == 6 && row.completed));
        assert!(!model.iter().any(|row| row.key == 1));
        assert_ne!(model_checksum(&model), 0);
    }

    #[test]
    fn timing_summary_uses_deterministic_median_and_p95() {
        let summary = timing_summary(vec![5.0, 1.0, 4.0, 2.0, 3.0]);
        assert_eq!(summary.samples, 5);
        assert_eq!(summary.median_ms, 3.0);
        assert_eq!(summary.p95_ms, 5.0);
    }
}
