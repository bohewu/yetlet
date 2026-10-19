//! Small, tab-local model. No storage, server settings or network capabilities.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub completed: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Filter {
    #[default]
    All,
    Active,
    Completed,
}

#[derive(Clone, Debug)]
pub struct Tasks {
    pub items: Vec<Task>,
    pub filter: Filter,
    next_id: u32,
}

#[derive(Clone, Debug)]
pub enum Command {
    Add(String),
    Rename(u32, String),
    Complete(u32, bool),
    Filter(Filter),
}

impl Default for Tasks {
    fn default() -> Self {
        Self {
            items: [
                "Plan the next small slice",
                "Keep browser configuration public",
                "Build something useful",
            ]
            .into_iter()
            .enumerate()
            .map(|(index, title)| Task {
                id: index as u32 + 1,
                title: title.into(),
                completed: index == 2,
            })
            .collect(),
            filter: Filter::All,
            next_id: 4,
        }
    }
}

impl Tasks {
    pub fn visible(&self) -> Vec<Task> {
        self.items
            .iter()
            .filter(|task| match self.filter {
                Filter::All => true,
                Filter::Active => !task.completed,
                Filter::Completed => task.completed,
            })
            .cloned()
            .collect()
    }

    pub fn apply(&mut self, command: Command) -> Result<(), String> {
        fn title(value: String) -> Result<String, String> {
            let value = value.trim();
            if value.is_empty() || value.chars().count() > 120 {
                return Err("Use a task title between 1 and 120 characters.".into());
            }
            Ok(value.into())
        }
        match command {
            Command::Add(value) => {
                let title = title(value)?;
                if self.items.len() >= 200 {
                    return Err("This small workspace holds up to 200 tasks.".into());
                }
                let id = self.next_id;
                let next = id.checked_add(1).ok_or("Task identity exhausted.")?;
                self.items.push(Task {
                    id,
                    title,
                    completed: false,
                });
                self.next_id = next;
            }
            Command::Rename(id, value) => {
                let title = title(value)?;
                self.items
                    .iter_mut()
                    .find(|task| task.id == id)
                    .ok_or("Task no longer exists.")?
                    .title = title;
            }
            Command::Complete(id, completed) => {
                self.items
                    .iter_mut()
                    .find(|task| task.id == id)
                    .ok_or("Task no longer exists.")?
                    .completed = completed;
            }
            Command::Filter(filter) => self.filter = filter,
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn local_commands_preserve_identity_and_filtered_authoritative_state() {
        let mut tasks = Tasks::default();
        tasks
            .apply(Command::Add("  新工作 <script>  ".into()))
            .unwrap();
        tasks.apply(Command::Rename(4, "修正工作".into())).unwrap();
        tasks.apply(Command::Complete(4, true)).unwrap();
        tasks.apply(Command::Filter(Filter::Completed)).unwrap();
        assert_eq!(
            tasks
                .visible()
                .iter()
                .map(|task| task.id)
                .collect::<Vec<_>>(),
            [3, 4]
        );
        assert_eq!(tasks.items[3].title, "修正工作");
        tasks.apply(Command::Filter(Filter::Active)).unwrap();
        assert_eq!(tasks.visible().len(), 2);
    }
    #[test]
    fn invalid_edits_and_capacity_do_not_change_existing_tasks() {
        let mut tasks = Tasks::default();
        let before = tasks.items.clone();
        for command in [
            Command::Add(" ".into()),
            Command::Rename(1, "x".repeat(121)),
            Command::Rename(99, "unknown".into()),
            Command::Complete(99, true),
        ] {
            assert!(tasks.apply(command).is_err());
            assert_eq!(tasks.items, before);
        }
        for _ in 3..200 {
            tasks.apply(Command::Add("task".into())).unwrap();
        }
        assert!(tasks.apply(Command::Add("overflow".into())).is_err());
        assert_eq!(tasks.items.len(), 200);
    }
}
