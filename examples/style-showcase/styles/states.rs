use crate::builder::{Block::*, Color, Declaration::*, Module, Rule, Weight};
use crate::style_names::*;

pub fn styles() -> Module {
    Module {
        name: "states",
        blocks: vec![
            Base(Rule::new(
                SELECTED,
                vec![Color(Color::Var("--app-accent")), FontWeight(Weight::Bold)],
            )),
            Base(Rule::new(
                UNSELECTED,
                vec![Color(Color::Var("--app-ink")), FontWeight(Weight::Normal)],
            )),
        ],
    }
}
