//! App-owned triage contract. No database, session, secret or server capability.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use yetlet_actions::{ActionError, Command};

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

// This wire payload belongs to this application, not the common action API.
pub type FieldErrors = BTreeMap<String, String>;
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationEnvelope {
    pub error: ValidationBody,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationBody {
    pub code: String,
    pub message: String,
    pub fields: FieldErrors,
}
fn valid_fields(fields: &FieldErrors) -> bool {
    !fields.is_empty()
        && fields.len() <= 5
        && fields.iter().all(|(name, message)| {
            matches!(
                name.as_str(),
                "title" | "description" | "status" | "priority" | "assignee"
            ) && !message.is_empty()
                && message.chars().count() <= 200
                && !message.chars().any(char::is_control)
        })
}
pub fn validation_fields(status: u16, body: &str) -> Option<FieldErrors> {
    if status != 422 {
        return None;
    }
    let payload: ValidationEnvelope = serde_json::from_str(body).ok()?;
    (payload.error.code == "validation" && valid_fields(&payload.error.fields))
        .then_some(payload.error.fields)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveFeedback {
    Loaded,
    Saved,
    Rejected,
    Access,
    Conflict,
    Unconfirmed,
}
impl SaveFeedback {
    pub fn from_error(error: &ActionError) -> Self {
        match error {
            ActionError::Remote {
                status: 401 | 403, ..
            } => Self::Access,
            ActionError::Remote { status: 409, .. } => Self::Conflict,
            ActionError::Remote { status: 422, .. } => Self::Rejected,
            _ => Self::Unconfirmed,
        }
    }
    pub fn needs_fresh_page(self) -> bool {
        matches!(self, Self::Access | Self::Conflict | Self::Unconfirmed)
    }
    pub fn guidance(self) -> &'static str {
        match self {
            Self::Access => {
                "Permission or session changed. Copy your draft, sign in in another tab, then open a fresh workspace and review your access before reapplying edits."
            }
            Self::Conflict => {
                "Conflict: the stored record changed. Copy your draft and compare the latest record in another tab, then reapply your edits in a fresh workspace."
            }
            Self::Unconfirmed => {
                "Save could not be confirmed; it may have reached the server. Copy your draft and check the latest record in another tab before reapplying edits in a fresh workspace."
            }
            Self::Rejected => {
                "Draft rejected. Check the highlighted fields and status transition. Your draft is still here."
            }
            _ => "",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Draft {
    pub saved: Item,
    pub value: Item,
    pub errors: FieldErrors,
    pub feedback: SaveFeedback,
}
impl Draft {
    pub fn dirty(&self) -> bool {
        self.value != self.saved
    }
    pub fn can_save(&self) -> bool {
        self.dirty() && !self.feedback.needs_fresh_page()
    }
    pub fn save_status(&self, pending: bool) -> &'static str {
        if pending {
            return "Saving";
        }
        match self.feedback {
            SaveFeedback::Access => "Access needs review",
            SaveFeedback::Conflict => "Conflict needs review",
            SaveFeedback::Unconfirmed => "Save unconfirmed",
            SaveFeedback::Rejected => "Needs corrections",
            _ if !self.errors.is_empty() => "Needs corrections",
            _ if self.dirty() => "Unsaved changes",
            SaveFeedback::Saved => "Saved",
            _ => "Loaded version",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Workspace {
    pub drafts: Vec<Draft>,
    pub selected: Option<i64>,
    pub query: String,
    pub pending: Option<i64>,
    pub description_reset: u32,
    pub message: String,
}
#[derive(Clone, Debug)]
pub enum Msg {
    Select(i64),
    Filter(String),
    Reverse,
    Edit { field: &'static str, value: String },
    EditDescription { id: i64, value: String },
    Reset,
    Begin,
    Saved(Item),
    Failed { id: i64, feedback: SaveFeedback },
    ValidationFailed { id: i64, fields: FieldErrors },
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
                    errors: FieldErrors::new(),
                    feedback: SaveFeedback::Loaded,
                })
                .collect(),
            query: String::new(),
            pending: None,
            description_reset: 0,
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
                draft.errors.remove(field);
                if draft.feedback == SaveFeedback::Rejected {
                    draft.feedback = SaveFeedback::Loaded;
                }
            }
            Msg::EditDescription { id, value } => {
                if self.selected != Some(id) || self.pending == Some(id) {
                    return Ok(next);
                }
                if value.chars().count() > 4000 {
                    return Err("Description exceeds 4,000 characters");
                }
                return self.reduce(Msg::Edit {
                    field: "description",
                    value,
                });
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
                    if draft.feedback.needs_fresh_page() {
                        return Ok(next);
                    }
                    draft.value = draft.saved.clone();
                    draft.errors.clear();
                    draft.feedback = SaveFeedback::Loaded;
                    next.description_reset = next.description_reset.wrapping_add(1);
                    next.message =
                        "Selected draft reset to its loaded version. Other drafts are unchanged."
                            .into();
                }
            }
            Msg::Begin => {
                if next.pending.is_some() || !next.selected().is_some_and(Draft::can_save) {
                    return Err("Nothing to save");
                }
                next.pending = next.selected;
                if let Some(draft) = next
                    .drafts
                    .iter_mut()
                    .find(|d| Some(d.value.id) == next.pending)
                {
                    draft.errors.clear();
                }
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
                draft.errors.clear();
                draft.feedback = SaveFeedback::Saved;
                next.pending = None;
                next.message = format!(
                    "Item #{} saved. Other drafts are still local.",
                    draft.value.id
                );
            }
            Msg::Failed { id, feedback } => {
                if next.pending != Some(id)
                    || matches!(feedback, SaveFeedback::Loaded | SaveFeedback::Saved)
                {
                    return Err("Unexpected save failure");
                }
                let draft = next
                    .drafts
                    .iter_mut()
                    .find(|d| d.value.id == id)
                    .ok_or("Missing draft")?;
                draft.feedback = feedback;
                next.pending = None;
                next.message = format!("Item #{id}: {}", feedback.guidance());
            }
            Msg::ValidationFailed { id, fields } => {
                if next.pending != Some(id) || !valid_fields(&fields) {
                    return Err("Unexpected validation response");
                }
                let draft = next
                    .drafts
                    .iter_mut()
                    .find(|d| d.value.id == id)
                    .ok_or("Missing draft")?;
                draft.errors = fields;
                draft.feedback = SaveFeedback::Rejected;
                next.pending = None;
                next.message = if next.selected == Some(id) {
                    "Check the highlighted fields. Your draft is still here.".into()
                } else {
                    format!("Item #{id} needs corrections. Select it to review its field messages.")
                };
            }
        }
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validation_payload_is_specific_bounded_and_plain_text() {
        let body = serde_json::json!({"error":{"code":"validation","message":"Check fields","fields":{"title":"Enter a title.","description":"<b>Plain text</b>"}}}).to_string();
        let fields = validation_fields(422, &body).unwrap();
        assert_eq!(fields["description"], "<b>Plain text</b>");
        for status in [200, 400, 401, 403, 409, 503] {
            assert!(validation_fields(status, &body).is_none());
        }
        for fields in [
            serde_json::json!({}),
            serde_json::json!({"unknown_field":"hidden"}),
            serde_json::json!({"title":"x".repeat(201)}),
            serde_json::json!({"title":"bad\nmessage"}),
        ] {
            let body = serde_json::json!({"error":{"code":"validation","message":"Check fields","fields":fields}}).to_string();
            assert!(validation_fields(422, &body).is_none());
        }
        assert!(validation_fields(422, "<html>denied</html>").is_none());
        assert!(validation_fields(422, &body.replace("validation", "other")).is_none());
        assert!(
            validation_fields(
                422,
                r#"{"error":{"code":"validation","message":"Check fields"}}"#
            )
            .is_none()
        );
    }
    #[test]
    fn field_errors_follow_submitted_draft_and_clear_only_when_obsolete() {
        let pending = state()
            .reduce(Msg::Edit {
                field: "title",
                value: "".into(),
            })
            .unwrap()
            .reduce(Msg::Begin)
            .unwrap();
        let fields = FieldErrors::from([
            ("title".into(), "Enter a title.".into()),
            ("description".into(), "Check description.".into()),
        ]);
        assert!(
            pending
                .reduce(Msg::ValidationFailed {
                    id: 2,
                    fields: fields.clone()
                })
                .is_err()
        );
        let state = pending
            .reduce(Msg::Select(2))
            .unwrap()
            .reduce(Msg::Edit {
                field: "title",
                value: "Other draft".into(),
            })
            .unwrap()
            .reduce(Msg::ValidationFailed {
                id: 1,
                fields: fields.clone(),
            })
            .unwrap();
        assert_eq!(state.selected, Some(2));
        assert!(state.selected().unwrap().errors.is_empty());
        assert_eq!(state.drafts[0].errors, fields);
        assert_eq!(state.drafts[0].value.title, "");
        assert!(state.pending.is_none());
        let state = state
            .reduce(Msg::Select(1))
            .unwrap()
            .reduce(Msg::Edit {
                field: "title",
                value: "Fixed".into(),
            })
            .unwrap();
        assert!(!state.selected().unwrap().errors.contains_key("title"));
        assert!(state.selected().unwrap().errors.contains_key("description"));
        let reset = state.reduce(Msg::Reset).unwrap();
        assert!(reset.selected().unwrap().errors.is_empty());
        assert_eq!(reset.drafts[1].value.title, "Other draft");
        let saving = state.reduce(Msg::Begin).unwrap();
        assert!(saving.selected().unwrap().errors.is_empty());
        let mut saved = saving.selected().unwrap().value.clone();
        saved.version += 1;
        let saved = saving.reduce(Msg::Saved(saved)).unwrap();
        assert!(saved.selected().unwrap().errors.is_empty());
        assert!(
            saved
                .reduce(Msg::ValidationFailed { id: 1, fields })
                .is_err()
        );
    }
    #[test]
    fn editor_updates_are_selected_bounded_and_blocked_while_saving() {
        let state = state();
        let edit = |id, value: &str| Msg::EditDescription {
            id,
            value: value.into(),
        };
        assert_eq!(
            state
                .reduce(edit(2, "old item callback"))
                .unwrap()
                .dirty_count(),
            0
        );
        let text = "😀".repeat(4000);
        let state = state.reduce(edit(1, &text)).unwrap();
        assert_eq!(state.selected().unwrap().value.description, text);
        assert!(state.reduce(edit(1, &"x".repeat(4001))).is_err());
        let pending = state.reduce(Msg::Begin).unwrap();
        assert_eq!(
            pending
                .reduce(edit(1, "late edit"))
                .unwrap()
                .selected()
                .unwrap()
                .value
                .description,
            text
        );
        let reset = state.reduce(Msg::Reset).unwrap();
        assert!(reset.selected().unwrap().value.description.is_empty());
        assert_eq!(reset.description_reset, 1);
        let status_only = reset
            .reduce(Msg::Edit {
                field: "status",
                value: "blocked".into(),
            })
            .unwrap();
        let reset_again = status_only.reduce(Msg::Reset).unwrap();
        assert_eq!(
            reset_again.selected().unwrap().value.description,
            reset.selected().unwrap().value.description
        );
        assert_eq!(reset_again.description_reset, 2);
    }
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
            .reduce(Msg::Failed {
                id: 1,
                feedback: SaveFeedback::Conflict,
            })
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

    #[test]
    fn save_status_tracks_edit_pending_success_and_correction() {
        let state = state();
        assert_eq!(
            state.selected().unwrap().save_status(false),
            "Loaded version"
        );
        let edited = state
            .reduce(Msg::Edit {
                field: "title",
                value: "New title".into(),
            })
            .unwrap();
        assert_eq!(
            edited.selected().unwrap().save_status(false),
            "Unsaved changes"
        );
        let saving = edited.reduce(Msg::Begin).unwrap();
        assert!(saving.reduce(Msg::Begin).is_err());
        assert_eq!(saving.selected().unwrap().save_status(true), "Saving");
        let mut item = saving.selected().unwrap().value.clone();
        item.version += 1;
        let saved = saving.reduce(Msg::Saved(item)).unwrap();
        assert_eq!(saved.selected().unwrap().save_status(false), "Saved");
        assert!(!saved.selected().unwrap().can_save());
        let rejected = edited
            .reduce(Msg::Begin)
            .unwrap()
            .reduce(Msg::Failed {
                id: 1,
                feedback: SaveFeedback::Rejected,
            })
            .unwrap();
        assert_eq!(
            rejected.selected().unwrap().save_status(false),
            "Needs corrections"
        );
        let corrected = rejected
            .reduce(Msg::Edit {
                field: "title",
                value: "Corrected".into(),
            })
            .unwrap();
        assert_eq!(
            corrected.selected().unwrap().save_status(false),
            "Unsaved changes"
        );
        assert!(corrected.reduce(Msg::Begin).is_ok());
        let reset = corrected.reduce(Msg::Reset).unwrap();
        assert_eq!(
            reset.selected().unwrap().save_status(false),
            "Loaded version"
        );
    }

    #[test]
    fn recovery_is_bound_to_failed_draft_and_blocks_replay_without_losing_edits() {
        for feedback in [
            SaveFeedback::Access,
            SaveFeedback::Conflict,
            SaveFeedback::Unconfirmed,
        ] {
            let saving = state()
                .reduce(Msg::Edit {
                    field: "title",
                    value: "Keep my work".into(),
                })
                .unwrap()
                .reduce(Msg::Begin)
                .unwrap();
            assert!(saving.reduce(Msg::Failed { id: 2, feedback }).is_err());
            let other = saving
                .reduce(Msg::Select(2))
                .unwrap()
                .reduce(Msg::Edit {
                    field: "title",
                    value: "Other draft".into(),
                })
                .unwrap();
            let failed = other.reduce(Msg::Failed { id: 1, feedback }).unwrap();
            assert_eq!(failed.selected().unwrap().feedback, SaveFeedback::Loaded);
            assert!(failed.selected().unwrap().can_save());
            let original = failed.reduce(Msg::Select(1)).unwrap();
            assert_eq!(original.selected().unwrap().value.title, "Keep my work");
            assert_eq!(original.selected().unwrap().feedback, feedback);
            assert!(original.message.contains("Item #1"));
            assert!(original.reduce(Msg::Begin).is_err());
            let reset = original.reduce(Msg::Reset).unwrap();
            assert_eq!(reset.selected().unwrap().value.title, "Keep my work");
            let edited = reset
                .reduce(Msg::Edit {
                    field: "description",
                    value: "More work".into(),
                })
                .unwrap();
            assert!(edited.reduce(Msg::Begin).is_err());
            assert_eq!(edited.selected().unwrap().saved.version, 1);
            assert_eq!(edited.drafts[1].value.title, "Other draft");
            assert!(edited.reduce(Msg::Failed { id: 1, feedback }).is_err());
        }
    }

    #[test]
    fn structured_failures_select_fixed_recovery_without_raw_remote_text() {
        for (status, expected) in [
            (401, SaveFeedback::Access),
            (403, SaveFeedback::Access),
            (409, SaveFeedback::Conflict),
            (422, SaveFeedback::Rejected),
            (503, SaveFeedback::Unconfirmed),
        ] {
            let feedback = SaveFeedback::from_error(&ActionError::Remote {
                status,
                code: "untrusted-response-content".into(),
            });
            assert_eq!(feedback, expected);
            assert!(!feedback.guidance().contains("untrusted-response-content"));
        }
        assert_eq!(
            SaveFeedback::from_error(&ActionError::Network),
            SaveFeedback::Unconfirmed
        );
        assert_eq!(
            SaveFeedback::from_error(&ActionError::InvalidResponse),
            SaveFeedback::Unconfirmed
        );
    }
}
