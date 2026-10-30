//! State for this example's one explicit remote create and read-only recovery.
use yetlet::actions::ActionError;
use yetlet_todo_ui::{MAX_TITLE_LENGTH, MAX_TODOS, TodoSnapshot};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum Phase {
    #[default]
    Ready,
    Saving,
    Unconfirmed,
    Reviewing,
    Reviewed,
}

#[derive(Default)]
pub(super) struct RemoteForm {
    pub phase: Phase,
    saved_title: Option<String>,
}
impl RemoteForm {
    pub fn can_create(&self, title: &str) -> bool {
        self.phase == Phase::Ready && self.saved_title.as_deref() != Some(title)
    }
    pub fn begin(&mut self, title: &str) -> bool {
        if !self.can_create(title) {
            return false;
        }
        self.phase = Phase::Saving;
        true
    }
    pub fn saved(&mut self, title: String) {
        self.saved_title = Some(title);
        self.phase = Phase::Ready;
    }
    pub fn rejected(&mut self) {
        self.phase = Phase::Ready;
    }
    pub fn unconfirmed(&mut self) {
        self.phase = Phase::Unconfirmed;
    }
    pub fn begin_review(&mut self) -> bool {
        if !matches!(self.phase, Phase::Unconfirmed | Phase::Reviewed) {
            return false;
        }
        self.phase = Phase::Reviewing;
        true
    }
    pub fn reviewed(&mut self, success: bool) {
        self.phase = if success {
            Phase::Reviewed
        } else {
            Phase::Unconfirmed
        };
    }
    pub fn resume(&mut self) -> bool {
        if self.phase != Phase::Reviewed {
            return false;
        }
        self.phase = Phase::Ready;
        self.saved_title = None;
        true
    }
}

pub(super) fn decode_snapshot(text: &str) -> Result<TodoSnapshot, ActionError> {
    let snapshot: TodoSnapshot =
        serde_json::from_str(text).map_err(|_| ActionError::InvalidResponse)?;
    let mut ids = std::collections::BTreeSet::new();
    if snapshot.items.len() > MAX_TODOS
        || snapshot.items.iter().any(|item| {
            item.id == 0
                || !ids.insert(item.id)
                || item.title.trim().is_empty()
                || item.title.chars().count() > MAX_TITLE_LENGTH
                || item.title.chars().any(char::is_control)
        })
    {
        return Err(ActionError::InvalidResponse);
    }
    Ok(snapshot)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_pending_create_and_unchanged_success_cannot_be_submitted_twice() {
        let mut form = RemoteForm::default();
        assert!(form.begin("First"));
        assert!(!form.begin("First"));
        assert!(!form.begin("Edited while saving"));
        form.saved("First".into());
        assert!(!form.begin("First"));
        assert!(form.begin("Another"));
        form.rejected();
        assert!(form.begin("Corrected"));
    }

    #[test]
    fn uncertain_create_requires_successful_read_and_explicit_resume() {
        let mut form = RemoteForm::default();
        assert!(form.begin("Keep this title"));
        form.unconfirmed();
        assert!(!form.begin("Keep this title"));
        assert!(!form.begin("Edited title"));
        assert!(!form.resume());
        assert!(form.begin_review());
        assert!(!form.begin_review());
        form.reviewed(false);
        assert!(!form.resume());
        assert!(form.begin_review());
        form.reviewed(true);
        assert!(!form.begin("Edited title"));
        assert!(form.resume());
        assert!(form.begin("Deliberate new create"));
    }

    #[test]
    fn server_review_accepts_only_a_usable_todo_snapshot() {
        let snapshot = TodoSnapshot::initial();
        assert_eq!(
            decode_snapshot(&serde_json::to_string(&snapshot).unwrap()).unwrap(),
            snapshot
        );
        assert!(decode_snapshot("<html>unavailable</html>").is_err());
        let mut duplicate = snapshot.clone();
        duplicate.items.push(duplicate.items[0].clone());
        assert!(decode_snapshot(&serde_json::to_string(&duplicate).unwrap()).is_err());
    }
}
