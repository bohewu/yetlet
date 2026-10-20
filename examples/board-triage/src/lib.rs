//! App-owned triage contract. No database, session, secret or server capability.
use serde::{Deserialize, Serialize};
use yetlet_actions::Command;

#[cfg(target_arch = "wasm32")]
mod browser;

pub const MAX_ITEMS: usize = 12;
pub const STATUS_LABELS: [&str; 4] = ["Open", "Active", "Blocked", "Closed"];

/// Count known statuses across the loaded items, including local draft edits.
/// Invalid draft statuses stay available for server validation, not a new bucket.
pub fn status_counts<'a>(items: impl IntoIterator<Item = &'a Item>) -> [u32; 4] {
    let mut counts = [0; 4];
    for item in items {
        let index = match item.status.as_str() {
            "open" => 0,
            "active" => 1,
            "blocked" => 2,
            "closed" => 3,
            _ => continue,
        };
        counts[index] += 1;
    }
    counts
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Item {
    pub id: i64,
    pub version: i64,
    pub title: String,
    pub description: String,
    pub status: String,
    pub priority: String,
    pub assignee: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bootstrap {
    pub schema: u8,
    pub project: i64,
    pub items: Vec<Item>,
}
impl Bootstrap {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema != 1 || !matches!(self.project, 1 | 2) || self.items.len() > MAX_ITEMS {
            return Err("Unsupported triage snapshot");
        }
        let mut ids = std::collections::BTreeSet::new();
        for item in &self.items {
            if item.id <= 0
                || item.version <= 0
                || !ids.insert(item.id)
                || item.title.chars().count() > 160
                || item.description.chars().count() > 4000
                || !matches!(
                    item.status.as_str(),
                    "open" | "active" | "blocked" | "closed"
                )
                || !matches!(item.priority.as_str(), "low" | "normal" | "high")
                || (!item.assignee.is_empty()
                    && item.assignee != if self.project == 1 { "alice" } else { "bob" })
            {
                return Err("Invalid triage item");
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Save {
    pub csrf: String,
    pub item: Item,
}
impl Command for Save {
    type Response = Item;
}

#[derive(Clone, Debug)]
pub struct Draft {
    pub saved: Item,
    pub value: Item,
}
impl Draft {
    pub fn dirty(&self) -> bool {
        self.value != self.saved
    }
}

#[derive(Clone, Debug)]
pub struct Workspace {
    pub drafts: Vec<Draft>,
    pub selected: Option<i64>,
    pub query: String,
    pub pending: Option<i64>,
    pub message: String,
}
#[derive(Clone, Debug)]
pub enum Msg {
    Select(i64),
    Filter(String),
    Reverse,
    Edit { field: &'static str, value: String },
    Reset,
    Begin,
    Saved(Item),
    Failed(String),
}
impl Workspace {
    pub fn new(snapshot: Bootstrap) -> Result<Self, &'static str> {
        snapshot.validate()?;
        Ok(Self {
            selected: snapshot.items.first().map(|item| item.id),
            drafts: snapshot
                .items
                .into_iter()
                .map(|value| Draft {
                    saved: value.clone(),
                    value,
                })
                .collect(),
            query: String::new(),
            pending: None,
            message: "Rust workspace ready. Changes stay here until you save an item.".into(),
        })
    }
    pub fn selected(&self) -> Option<&Draft> {
        self.drafts
            .iter()
            .find(|d| Some(d.value.id) == self.selected)
    }
    pub fn visible(&self) -> Vec<Draft> {
        let query = self.query.to_lowercase();
        self.drafts
            .iter()
            .filter(|d| d.value.title.to_lowercase().contains(&query))
            .cloned()
            .collect()
    }
    pub fn dirty_count(&self) -> usize {
        self.drafts.iter().filter(|d| d.dirty()).count()
    }
    pub fn reduce(&self, msg: Msg) -> Result<Self, &'static str> {
        let mut next = self.clone();
        match msg {
            Msg::Select(id) => {
                if !next.drafts.iter().any(|d| d.value.id == id) {
                    return Err("Unknown item");
                }
                next.selected = Some(id);
                next.message = format!("Item #{id} selected. Unsaved drafts remain in this tab.");
            }
            Msg::Filter(query) => next.query = query,
            Msg::Reverse => next.drafts.reverse(),
            Msg::Edit { field, value } => {
                if next.selected == next.pending {
                    return Ok(next);
                }
                let draft = next
                    .drafts
                    .iter_mut()
                    .find(|d| Some(d.value.id) == next.selected)
                    .ok_or("No selection")?;
                match field {
                    "title" => draft.value.title = value,
                    "description" => draft.value.description = value,
                    "status" => draft.value.status = value,
                    "priority" => draft.value.priority = value,
                    "assignee" => draft.value.assignee = value,
                    _ => return Err("Unknown field"),
                }
            }
            Msg::Reset => {
                if next.selected == next.pending {
                    return Ok(next);
                }
                if let Some(draft) = next
                    .drafts
                    .iter_mut()
                    .find(|d| Some(d.value.id) == next.selected)
                {
                    draft.value = draft.saved.clone();
                    next.message =
                        "Selected draft reset to its loaded version. Other drafts are unchanged."
                            .into();
                }
            }
            Msg::Begin => {
                if next.pending.is_some() || !next.selected().is_some_and(Draft::dirty) {
                    return Err("Nothing to save");
                }
                next.pending = next.selected;
                next.message = "Saving selected item… You can work on another draft.".into();
            }
            Msg::Saved(item) => {
                if next.pending != Some(item.id) {
                    return Err("Unexpected save response");
                }
                let draft = next
                    .drafts
                    .iter_mut()
                    .find(|d| d.value.id == item.id)
                    .ok_or("Missing saved item")?;
                if Some(item.version) != draft.saved.version.checked_add(1) {
                    return Err("Unexpected saved version");
                }
                draft.saved = item.clone();
                draft.value = item;
                next.pending = None;
                next.message = "Item saved. Other drafts are still local.".into();
            }
            Msg::Failed(message) => {
                next.pending = None;
                next.message = message;
            }
        }
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn item(id: i64) -> Item {
        Item {
            id,
            version: 1,
            title: format!("Item {id}"),
            description: String::new(),
            status: "open".into(),
            priority: "normal".into(),
            assignee: String::new(),
        }
    }
    fn state() -> Workspace {
        Workspace::new(Bootstrap {
            schema: 1,
            project: 1,
            items: vec![item(1), item(2)],
        })
        .unwrap()
    }
    #[test]
    fn counts_cover_all_loaded_drafts_and_follow_reset() {
        let state = state()
            .reduce(Msg::Edit {
                field: "status",
                value: "blocked".into(),
            })
            .unwrap()
            .reduce(Msg::Filter("no matching title".into()))
            .unwrap();
        assert!(state.visible().is_empty());
        assert_eq!(
            status_counts(state.drafts.iter().map(|d| &d.value)),
            [1, 0, 1, 0]
        );
        let state = state.reduce(Msg::Reset).unwrap();
        assert_eq!(
            status_counts(state.drafts.iter().map(|d| &d.value)),
            [2, 0, 0, 0]
        );
        assert_eq!(status_counts([].iter()), [0; 4]);
        let mut items = [item(1), item(2), item(3), item(4)];
        for (item, status) in items
            .iter_mut()
            .zip(["open", "active", "blocked", "closed"])
        {
            item.status = status.into();
        }
        assert_eq!(status_counts(&items), [1; 4]);
        items[0].status = "invalid draft".into();
        assert_eq!(status_counts(&items), [0, 1, 1, 1]);
    }
    #[test]
    fn drafts_survive_switch_filter_and_reorder() {
        let state = state()
            .reduce(Msg::Edit {
                field: "title",
                value: "中文 draft".into(),
            })
            .unwrap()
            .reduce(Msg::Select(2))
            .unwrap()
            .reduce(Msg::Reverse)
            .unwrap()
            .reduce(Msg::Filter("Item".into()))
            .unwrap();
        assert_eq!(state.visible().len(), 1);
        assert_eq!(state.dirty_count(), 1);
        assert_eq!(
            state
                .reduce(Msg::Select(1))
                .unwrap()
                .selected()
                .unwrap()
                .value
                .title,
            "中文 draft"
        );
    }
    #[test]
    fn pending_save_protects_its_draft_and_does_not_overwrite_another_selection() {
        let state = state()
            .reduce(Msg::Edit {
                field: "title",
                value: "Saved title".into(),
            })
            .unwrap()
            .reduce(Msg::Begin)
            .unwrap();
        let state = state
            .reduce(Msg::Edit {
                field: "title",
                value: "ignored".into(),
            })
            .unwrap();
        let mut saved = state.selected().unwrap().value.clone();
        saved.version = 2;
        let state = state
            .reduce(Msg::Select(2))
            .unwrap()
            .reduce(Msg::Edit {
                field: "description",
                value: "Other draft".into(),
            })
            .unwrap()
            .reduce(Msg::Saved(saved))
            .unwrap();
        assert_eq!(state.selected, Some(2));
        assert_eq!(state.dirty_count(), 1);
        assert_eq!(state.drafts[0].saved.version, 2);
        assert!(state.pending.is_none());
        assert!(state.reduce(Msg::Saved(item(1))).is_err());
    }
    #[test]
    fn failed_save_preserves_draft_and_rejects_duplicate_bootstrap() {
        let state = state()
            .reduce(Msg::Edit {
                field: "title",
                value: "Keep me".into(),
            })
            .unwrap()
            .reduce(Msg::Begin)
            .unwrap()
            .reduce(Msg::Failed("Conflict".into()))
            .unwrap();
        assert_eq!(state.selected().unwrap().value.title, "Keep me");
        assert_eq!(state.selected().unwrap().saved.version, 1);
        assert!(
            Workspace::new(Bootstrap {
                schema: 1,
                project: 1,
                items: vec![item(1), item(1)]
            })
            .is_err()
        );
    }
}
