mod components;
mod layout;
mod states;
mod tokens;
use crate::builder::{StyleError, StyleSheet};

// Every module is explicitly registered, including classes absent in initial View.
pub fn stylesheet() -> Result<StyleSheet, StyleError> {
    StyleSheet::compose(vec![
        tokens::styles(),
        layout::styles(),
        components::styles(),
        states::styles(),
    ])
}
