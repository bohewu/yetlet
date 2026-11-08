#[path = "layout.rs"]
mod layout;
#[path = "tasks.rs"]
mod tasks;
use crate::builder::{StyleError, StyleSheet};

// Registration is explicit, independent of which View branches execute.
pub fn stylesheet() -> Result<StyleSheet, StyleError> {
    StyleSheet::compose(vec![layout::styles(), tasks::styles()])
}
