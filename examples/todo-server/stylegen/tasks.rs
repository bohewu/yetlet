use crate::builder::{Block::*, Color, Declaration::*, Length::*, Module, Rule};
use crate::style_names::*;
pub fn styles() -> Module {
    Module {
        name: "tasks",
        blocks: vec![
            Base(Rule::new(LIST, vec![Padding(Px(0.0))])),
            Base(Rule::new(OPEN, vec![Color(Color::Var("--app-ink"))])),
            // No initial Todo is complete. This rule still ships before any toggle.
            Base(Rule::new(
                COMPLETE,
                vec![Color(Color::Var("--app-complete"))],
            )),
        ],
    }
}
