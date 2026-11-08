#[path = "dialog.rs"]
mod dialog;
#[path = "layout.rs"]
mod layout;

use crate::builder::{StyleError, StyleSheet};

// Already wired: adding a source file alone does not register a module.
pub fn stylesheet() -> Result<StyleSheet, StyleError> {
    StyleSheet::compose(vec![layout::styles(), dialog::styles()])
}
